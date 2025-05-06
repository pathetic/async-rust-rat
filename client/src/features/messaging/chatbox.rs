use std::ffi::{OsStr, OsString};
use std::mem;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::ptr::{null, null_mut};
use winapi::shared::minwindef::{LPARAM, LRESULT, TRUE, UINT, WPARAM};
use winapi::shared::windef::{HBRUSH, HMENU, HWND};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::{CreateFontW, ANSI_CHARSET, DEFAULT_PITCH, DEFAULT_QUALITY, FF_DONTCARE, FW_NORMAL, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS};
use winapi::um::winuser::*;
use winapi::shared::winerror::*;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winuser::IsWindow;
use crate::handler::send_packet;
use common::packets::ServerboundPacket;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const CHAT_HISTORY_ID: i32 = 100;
const INPUT_BOX_ID: i32 = 101;
const SEND_BUTTON_ID: i32 = 102;

// Global channel for sending chat messages
static MESSAGE_SENDER: Lazy<Mutex<Option<mpsc::Sender<String>>>> = Lazy::new(|| {
    // Create the channel 
    let (tx, rx) = mpsc::channel::<String>();
    
    // Spawn a dedicated thread for handling message sending
    std::thread::spawn(move || {
        // Create a single tokio runtime on this dedicated thread
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        
        // Process messages from the channel without blocking
        while let Ok(message) = rx.recv() {
            println!("Sending chat message: {}", message);
            
            // Clone the message to avoid ownership issues
            let message_clone = message.clone();
            
            // Run the send_packet task on our dedicated runtime
            rt.block_on(async {
                if let Err(e) = send_packet(ServerboundPacket::ChatMessage(message_clone)).await {
                    println!("Failed to send chat message: {}", e);
                    eprintln!("Failed to send chat message: {}", e);
                } else {
                    println!("Chat message sent successfully");
                }
            });
        }
    });
    
    Mutex::new(Some(tx))
});

// Static UI message sender that persists throughout the program
static UI_SENDER: Lazy<Mutex<Option<mpsc::Sender<UiMessage>>>> = Lazy::new(|| {
    Mutex::new(None)
});

// Track if the UI thread is already running
static UI_THREAD_RUNNING: Lazy<Arc<AtomicBool>> = Lazy::new(|| {
    Arc::new(AtomicBool::new(false))
});

// Messages that can be sent to the UI thread
enum UiMessage {
    CreateWindow,
    AddMessage { sender: String, message: String },
    Close,
    EnsureWindowVisible,
    ForceForeground,
    ForceClose,  // New forceful close message
}

struct ChatMessages {
    messages: Vec<(String, String)>,
}

pub struct ChatBoxState {
    hwnd: HWND,
    messages: Vec<(String, String)>,
    ui_sender: Option<mpsc::Sender<UiMessage>>,
}

impl ChatBoxState {
    pub fn new() -> Self {
        // Start the UI thread if it's not already running
        if !UI_THREAD_RUNNING.load(Ordering::SeqCst) {
            start_ui_thread();
        }
        
        // Get the sender to communicate with the UI thread
        let ui_sender = UI_SENDER.lock().unwrap().clone();
        
        Self {
            hwnd: null_mut(),
            messages: Vec::new(),
            ui_sender,
        }
    }
    
    // Only call this if we explicitly want to initialize the chatbox
    pub fn initialize(&mut self) {
        println!("Initializing chatbox...");
        if let Some(ui_sender) = &self.ui_sender {
            println!("UI sender found, sending initialization messages");
            
            // Explicitly create the window
            let result = ui_sender.send(UiMessage::CreateWindow);
            println!("CreateWindow initialization message sent: {:?}", result);
            
            // Add a welcome message
            println!("Adding welcome message");
            self.add_message_internal("System", "Chat initialized. Messages will appear here.");
        } else {
            println!("Failed to initialize chatbox - no UI sender available");
            
            // Try to recreate the UI thread and get a new sender
            if !UI_THREAD_RUNNING.load(Ordering::SeqCst) {
                println!("Attempting to restart UI thread");
                start_ui_thread();
                self.ui_sender = UI_SENDER.lock().unwrap().clone();
                
                if self.ui_sender.is_some() {
                    println!("Successfully reconnected to UI thread");
                    self.initialize(); // Try again
                } else {
                    println!("FATAL: Could not reconnect to UI thread");
                }
            } else {
                println!("UI thread is running but sender is not available");
            }
        }
        println!("Chatbox initialization completed");
    }

    // This is the main method called by external code
    pub fn add_message(&mut self, sender: &str, message: &str) {
        println!("ChatBoxState::add_message called with: {} - {}", sender, message);
        
        // Initialize the chatbox if this is the first message
        if self.messages.is_empty() {
            println!("First message received, initializing chatbox");
            // Add the message directly to our local state
            self.messages.push((sender.to_string(), message.to_string()));
            // Then initialize with this message
            self.initialize();
        } else {
            // Add the message normally
            self.add_message_internal(sender, message);
        }
    }
    
    // Internal helper to add a message without recursive initialization
    fn add_message_internal(&mut self, sender: &str, message: &str) {
        // Store the message locally
        self.messages.push((sender.to_string(), message.to_string()));
        
        // Send the message to the UI thread
        if let Some(ui_sender) = &self.ui_sender {
            println!("Sending CreateWindow message to UI thread");
            // First ensure the window is created if it doesn't exist
            let result = ui_sender.send(UiMessage::CreateWindow);
            println!("CreateWindow message sent: {:?}", result);
            
            println!("Sending AddMessage to UI thread");
            // Then send the message
            let result = ui_sender.send(UiMessage::AddMessage { 
                sender: sender.to_string(), 
                message: message.to_string() 
            });
            println!("AddMessage sent: {:?}", result);
            
            println!("Sending EnsureWindowVisible to UI thread");
            // Ensure the window is visible
            let result = ui_sender.send(UiMessage::EnsureWindowVisible);
            println!("EnsureWindowVisible sent: {:?}", result);
        } else {
            println!("ERROR: No UI sender available to send message");
            
            // Try to reconnect to UI thread
            if UI_THREAD_RUNNING.load(Ordering::SeqCst) {
                println!("UI thread is running, attempting to get sender");
                self.ui_sender = UI_SENDER.lock().unwrap().clone();
                
                if self.ui_sender.is_some() {
                    println!("Successfully reconnected to UI thread");
                    self.add_message_internal(sender, message); // Try again with the new sender
                    return;
                }
            }
            
            // If we still don't have a sender, try restarting the UI thread
            println!("Attempting to restart UI thread");
            start_ui_thread();
            self.ui_sender = UI_SENDER.lock().unwrap().clone();
            
            if self.ui_sender.is_some() {
                println!("Successfully reconnected to restarted UI thread");
                self.add_message_internal(sender, message); // Try again with the new sender
            } else {
                println!("FATAL: Could not reconnect to UI thread after restart");
            }
        }
    }

    pub fn close(&mut self) {
        println!("ChatBoxState::close called");
        
        if let Some(ui_sender) = &self.ui_sender {
            // Try a forceful direct close first
            println!("Sending ForceClose message to UI thread");
            let result = ui_sender.send(UiMessage::ForceClose);
            println!("ForceClose message sent: {:?}", result);
            
            // Small delay to let the forceful close complete
            std::thread::sleep(std::time::Duration::from_millis(200));
            
            // In case forceful close didn't work, try the normal close as a backup
            println!("Sending normal Close message as backup");
            let result = ui_sender.send(UiMessage::Close);
            println!("Close message sent: {:?}", result);
        } else {
            println!("No UI sender available to close window");
        }
        
        // Clear our local message store
        self.messages.clear();
        self.hwnd = null_mut();
    }
}

// Start the UI thread
fn start_ui_thread() {
    println!("start_ui_thread called");
    if UI_THREAD_RUNNING.load(Ordering::SeqCst) {
        println!("UI thread already running, not starting another one");
        return;
    }
    
    // Create a new channel
    let (tx, rx) = mpsc::channel();
    println!("Created new UI channel");
    
    // Store the sender
    {
        let mut sender_guard = UI_SENDER.lock().unwrap();
        *sender_guard = Some(tx);
        println!("Stored UI sender in static variable");
    }
    
    UI_THREAD_RUNNING.store(true, Ordering::SeqCst);
    println!("UI thread marked as running");
    
    // Start the UI thread
    println!("Spawning UI thread");
    std::thread::spawn(move || {
        println!("UI thread started");
        let mut ui_state = UiState::new();
        println!("UI state created");
        
        // Don't create the window immediately - wait for a message
        // println!("Creating window during UI thread initialization");
        // ui_state.create_window();
        
        let mut running = true;
        
        // Main message processing loop
        while running {
            // Process any application messages without blocking
            while let Ok(message) = rx.try_recv() {
                println!("UI thread received message: {:?}", message);
                match message {
                    UiMessage::CreateWindow => {
                        println!("Processing CreateWindow message");
                        if ui_state.hwnd.is_null() {
                            println!("Window handle is null, creating window");
                            ui_state.create_window();
                        } else {
                            println!("Window already exists, hwnd: {:?}", ui_state.hwnd);
                            ui_state.ensure_window_visible();
                        }
                    },
                    UiMessage::AddMessage { sender, message } => {
                        println!("Processing AddMessage: {} - {}", sender, message);
                        ui_state.add_message(&sender, &message);
                    },
                    UiMessage::Close => {
                        println!("Processing Close message");
                        // Force extra processing to ensure window is destroyed
                        if !ui_state.hwnd.is_null() {
                            println!("Closing window via Close message");
                            ui_state.close();
                            
                            // Process any window messages that might have been generated
                            ui_state.process_messages();
                            
                            println!("Closed window, verifying hwnd is null: {}", ui_state.hwnd.is_null());
                        } else {
                            println!("Window already closed (hwnd is null)");
                        }
                        
                        // Don't exit the thread - just close the window
                        // The thread should keep running to handle future messages
                    },
                    UiMessage::EnsureWindowVisible => {
                        println!("Processing EnsureWindowVisible message");
                        ui_state.ensure_window_visible();
                    },
                    UiMessage::ForceForeground => {
                        println!("Processing ForceForeground message");
                        if !ui_state.hwnd.is_null() {
                            ui_state.force_foreground();
                        }
                    },
                    UiMessage::ForceClose => {
                        println!("Processing ForceClose message");
                        if !ui_state.hwnd.is_null() {
                            println!("Force closing window");
                            ui_state.force_close();
                            println!("Force close completed");
                        } else {
                            println!("Window already closed (hwnd is null)");
                        }
                    },
                }
            }
            
            // Process Windows messages
            ui_state.process_messages();
            
            // Sleep a bit to avoid consuming 100% CPU
            std::thread::sleep(std::time::Duration::from_millis(10));
            
            // Check if we've been asked to exit during message processing
            if !running {
                break;
            }
            
            // Check for channel closed
            match rx.try_recv() {
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    println!("UI channel disconnected, exiting UI thread");
                    break;
                },
                _ => {} // Either got a message (already processed) or would block (expected)
            }
        }
        
        println!("UI thread exiting");
        UI_THREAD_RUNNING.store(false, Ordering::SeqCst);
        
        // Clear the sender when the thread exits
        let mut sender_guard = UI_SENDER.lock().unwrap();
        *sender_guard = None;
    });
    println!("UI thread spawned");
}

// State maintained by the UI thread
struct UiState {
    hwnd: HWND,
    messages: Vec<(String, String)>,
}

impl UiState {
    fn new() -> Self {
        Self {
            hwnd: null_mut(),
            messages: Vec::new(),
        }
    }
    
    fn create_window(&mut self) {
        if !self.hwnd.is_null() {
            // Window already exists, just make it visible
            println!("Window already exists (hwnd={:?}), making it visible", self.hwnd);
            self.ensure_window_visible();
            return;
        }
        
        println!("Creating chatbox window...");
        
        unsafe {
            let h_instance = GetModuleHandleW(null());
            
            let class_name = to_wstring("ChatBoxClass");
            
            // Check if class is already registered
            let mut wcx: WNDCLASSEXW = mem::zeroed();
            let class_exists = GetClassInfoExW(h_instance, class_name.as_ptr(), &mut wcx) != 0;
            
            if !class_exists {
                let wc = WNDCLASSEXW {
                    cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                    style: CS_HREDRAW | CS_VREDRAW,
                    lpfnWndProc: Some(window_proc),
                    cbClsExtra: 0,
                    cbWndExtra: 0,
                    hInstance: h_instance,
                    hIcon: null_mut(),
                    hCursor: LoadCursorW(null_mut(), IDC_ARROW),
                    hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
                    lpszMenuName: null(),
                    lpszClassName: class_name.as_ptr(),
                    hIconSm: null_mut(),
                };
                
                if RegisterClassExW(&wc) == 0 {
                    let error = GetLastError();
                    eprintln!("Failed to register window class, error code: {}", error);
                    return;
                } else {
                    println!("Chat window class registered successfully");
                }
            } else {
                println!("Chat window class already registered");
            }
            
            let window_name = to_wstring("Chat Box");
            
            // Create window with title bar and close button but no min/max buttons
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_name.as_ptr(),
                WS_CAPTION | WS_SYSMENU | WS_BORDER | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                600,
                500,
                null_mut(),
                null_mut(),
                h_instance,
                null_mut(),
            );
            
            if hwnd.is_null() {
                let error = GetLastError();
                eprintln!("Failed to create window, error code: {}", error);
                return;
            } else {
                println!("Chat window created successfully");
            }
            
            // Store window handle
            self.hwnd = hwnd;
            
            // Store the ChatBox instance in the window's user data
            let state = Box::new(ChatMessages {
                messages: Vec::new(),
            });
            let state_ptr = Box::into_raw(state);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
            
            // Create chat history text box
            let chat_hwnd = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                to_wstring("EDIT").as_ptr(),
                null(),
                WS_CHILD | WS_VISIBLE | WS_VSCROLL | ES_MULTILINE | ES_READONLY | ES_AUTOVSCROLL,
                10,
                10,
                560,
                350,
                hwnd,
                CHAT_HISTORY_ID as HMENU,
                h_instance,
                null_mut(),
            );
            
            if chat_hwnd.is_null() {
                let error = GetLastError();
                eprintln!("Failed to create chat history window, error code: {}", error);
            }
            
            // Set font for the chat history
            let hfont = CreateFontW(
                16, 0, 0, 0, FW_NORMAL, 0, 0, 0, 
                ANSI_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, 
                DEFAULT_QUALITY, DEFAULT_PITCH | FF_DONTCARE, 
                to_wstring("Consolas").as_ptr()
            );
            SendMessageW(chat_hwnd, WM_SETFONT, hfont as WPARAM, TRUE as LPARAM);
            
            // Create input box
            let input_hwnd = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                to_wstring("EDIT").as_ptr(),
                null(),
                WS_CHILD | WS_VISIBLE | ES_AUTOHSCROLL | ES_WANTRETURN,
                10,
                370,
                480,
                40,
                hwnd,
                INPUT_BOX_ID as HMENU,
                h_instance,
                null_mut(),
            );
            
            if input_hwnd.is_null() {
                let error = GetLastError();
                eprintln!("Failed to create input box, error code: {}", error);
            }
            
            SendMessageW(input_hwnd, WM_SETFONT, hfont as WPARAM, TRUE as LPARAM);
            
            // Create send button
            let button_hwnd = CreateWindowExW(
                0,
                to_wstring("BUTTON").as_ptr(),
                to_wstring("send >").as_ptr(),
                WS_VISIBLE | WS_CHILD | BS_DEFPUSHBUTTON,
                500,
                370,
                70,
                40,
                hwnd,
                SEND_BUTTON_ID as HMENU,
                h_instance,
                null_mut(),
            );
            
            if button_hwnd.is_null() {
                let error = GetLastError();
                eprintln!("Failed to create send button, error code: {}", error);
            }
            
            SendMessageW(button_hwnd, WM_SETFONT, hfont as WPARAM, TRUE as LPARAM);
            
            // Show the window
            ShowWindow(hwnd, SW_SHOWNORMAL);
            UpdateWindow(hwnd);
            
            // Focus on the input box
            SetFocus(input_hwnd);
            
            // Initialize with existing messages
            self.update_chat_display();
            
            // Process any pending messages to ensure window is properly created
            self.process_messages();
            
            println!("Chat window initialization complete");
        }
    }
    
    fn add_message(&mut self, sender: &str, message: &str) {
        if self.hwnd.is_null() {
            self.create_window();
        }

        self.messages.push((sender.to_string(), message.to_string()));
        self.update_chat_display();
    }
    
    fn update_chat_display(&self) {
        if self.hwnd.is_null() {
            return;
        }
        
        let mut chat_text = String::new();
        for (sender, message) in &self.messages {
            chat_text.push_str(&format!("{}: {}\r\n", sender, message));
        }

        unsafe {
            let chat_hwnd = GetDlgItem(self.hwnd, CHAT_HISTORY_ID);
            if chat_hwnd.is_null() {
                return;
            }
            
            let chat_wstr = to_wstring(&chat_text);
            SetWindowTextW(chat_hwnd, chat_wstr.as_ptr());
            
            // Scroll to bottom
            SendMessageW(chat_hwnd, EM_SETSEL as u32, 0, chat_text.len() as LPARAM);
            SendMessageW(chat_hwnd, EM_SETSEL as u32, chat_text.len() as WPARAM, chat_text.len() as LPARAM);
            SendMessageW(chat_hwnd, EM_SCROLLCARET as u32, 0, 0);
        }
    }

    fn close(&mut self) {
        if self.hwnd.is_null() {
            println!("Window already closed (null hwnd)");
            return;
        }
        
        println!("Closing chatbox window (hwnd={:?})", self.hwnd);
        
        unsafe {
            // Check if window is valid
            if IsWindow(self.hwnd) == 0 {
                println!("Window is not valid, reset hwnd and return");
                self.hwnd = null_mut();
                self.messages.clear();
                return;
            }
            
            // First hide the window to provide immediate visual feedback
            ShowWindow(self.hwnd, SW_HIDE);
            println!("Window hidden");
            
            // Use SendMessage for synchronous execution
            println!("Sending WM_CLOSE directly to window");
            SendMessageW(self.hwnd, WM_CLOSE, 0, 0);
            
            // Force window destruction with EndDialog as a fallback
            if IsWindow(self.hwnd) != 0 {  // If window still exists
                println!("Window still exists after WM_CLOSE, using DestroyWindow");
                DestroyWindow(self.hwnd);
            } else {
                println!("Window successfully closed with WM_CLOSE");
            }
            
            // Force cleanup of the HWND to ensure we don't try to use it again
            self.hwnd = null_mut();
            self.messages.clear();
            
            println!("Chatbox window close operations completed");
        }
    }
    
    fn process_messages(&self) {
        if self.hwnd.is_null() {
            // No window to process messages for
            return;
        }
        
        unsafe {
            let mut msg: MSG = mem::zeroed();
            let mut processed = 0;
            
            // Process all pending messages without blocking
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) > 0 {
                processed += 1;
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            
            if processed > 0 {
                println!("Processed {} Windows messages", processed);
            }
        }
    }
    
    fn run_message_loop(&self) {
        unsafe {
            let mut msg: MSG = mem::zeroed();
            
            // Standard Windows message loop - blocks until a message is received
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    fn ensure_window_visible(&self) {
        if self.hwnd.is_null() {
            println!("Cannot make window visible - hwnd is null");
            return;
        }
        
        println!("Making window visible and bringing to front (hwnd={:?})", self.hwnd);
        
        unsafe {
            // Bring the window to the front
            let result = SetForegroundWindow(self.hwnd);
            println!("SetForegroundWindow result: {}", result);
            
            // Make sure it's showing and not minimized
            ShowWindow(self.hwnd, SW_SHOWNORMAL);
            
            // Force the window to redraw
            UpdateWindow(self.hwnd);
            
            println!("Window should now be visible");
        }
    }

    fn force_foreground(&self) {
        if self.hwnd.is_null() {
            println!("Cannot bring window to foreground - hwnd is null");
            return;
        }
        
        println!("Forcing window to foreground (hwnd={:?})", self.hwnd);
        
        unsafe {
            // Try multiple approaches to ensure the window comes to foreground
            
            // First, check if window is valid
            if IsWindow(self.hwnd) == 0 {
                println!("Window is not valid");
                return;
            }
            
            // Try to bring to foreground
            let result = SetForegroundWindow(self.hwnd);
            println!("SetForegroundWindow result: {}", result);
            
            // Force activation
            ShowWindow(self.hwnd, SW_RESTORE);
            ShowWindow(self.hwnd, SW_SHOW);
            UpdateWindow(self.hwnd);
            
            // Set active
            SetActiveWindow(self.hwnd);
            
            println!("Window should now be in foreground");
        }
    }

    fn force_close(&mut self) {
        if self.hwnd.is_null() {
            println!("Cannot close window - hwnd is null");
            return;
        }
        
        println!("Force closing window (hwnd={:?})", self.hwnd);
        
        unsafe {
            // Check if window is valid
            if IsWindow(self.hwnd) == 0 {
                println!("Window is not valid, reset hwnd and return");
                self.hwnd = null_mut();
                return;
            }
            
            // Try EnumChildWindows to close all child windows
            let hwnd_copy = self.hwnd;
            EnumChildWindows(
                self.hwnd,
                Some(enum_child_proc),
                0
            );
            
            // Hide the window completely
            ShowWindow(self.hwnd, SW_HIDE);
            
            // Try all approaches to close the window
            PostMessageW(self.hwnd, WM_SYSCOMMAND, SC_CLOSE, 0);
            PostMessageW(self.hwnd, WM_CLOSE, 0, 0);
            DestroyWindow(self.hwnd);
            
            // Verify the window was destroyed
            if IsWindow(self.hwnd) != 0 {
                println!("WARNING: Window still exists after multiple close attempts");
                // Final desperate attempt
                SetWindowLongPtrW(self.hwnd, GWL_STYLE, 0);
                DestroyWindow(self.hwnd);
            } else {
                println!("Window successfully destroyed");
            }
            
            // Clear our reference regardless
            self.hwnd = null_mut();
            self.messages.clear();
        }
    }
}

fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

unsafe fn send_message(hwnd: HWND) {
    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ChatMessages;
    if state_ptr.is_null() {
        return;
    }
    
    let state = &mut *state_ptr;
    let input_hwnd = GetDlgItem(hwnd, INPUT_BOX_ID);
    
    let mut buffer = [0u16; 1024];
    let len = GetWindowTextW(input_hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
    if len > 0 {
        let message_text = OsString::from_wide(&buffer[..len as usize])
            .to_string_lossy()
            .into_owned();
        
        state.messages.push(("Client".to_string(), message_text.clone()));
        
        // Update chat display
        update_chat_display(hwnd, state);
        
        // Clear input box
        SetWindowTextW(input_hwnd, [0u16].as_ptr());
        
        // Send the message through our channel
        if let Some(sender) = MESSAGE_SENDER.lock().unwrap().as_ref() {
            if let Err(e) = sender.send(message_text) {
                eprintln!("Failed to send message to channel: {}", e);
            }
        }
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            // Initialize an empty state for the window
            let state = Box::new(ChatMessages {
                messages: Vec::new(),
            });
            let state_ptr = Box::into_raw(state);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
            0
        }
        
        WM_COMMAND => {
            if loword(wparam as u32) as i32 == SEND_BUTTON_ID && hiword(wparam as u32) == BN_CLICKED {
                send_message(hwnd);
            }
            0
        }
        
        WM_CHAR => {
            // Check if Enter key was pressed in the input box
            let focused_hwnd = GetFocus();
            let input_hwnd = GetDlgItem(hwnd, INPUT_BOX_ID);
            
            if focused_hwnd == input_hwnd && wparam == VK_RETURN as usize {
                send_message(hwnd);
                return 0;
            }
            
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        
        WM_SIZE => {
            let width = loword(lparam as u32) as i32;
            let height = hiword(lparam as u32) as i32;
            
            // Resize the chat history
            let chat_hwnd = GetDlgItem(hwnd, CHAT_HISTORY_ID);
            SetWindowPos(
                chat_hwnd,
                null_mut(),
                10,
                10,
                width - 20,
                height - 80,
                SWP_NOZORDER,
            );
            
            // Resize and reposition the input box
            let input_hwnd = GetDlgItem(hwnd, INPUT_BOX_ID);
            SetWindowPos(
                input_hwnd,
                null_mut(),
                10,
                height - 60,
                width - 100,
                40,
                SWP_NOZORDER,
            );
            
            // Reposition the send button
            let button_hwnd = GetDlgItem(hwnd, SEND_BUTTON_ID);
            SetWindowPos(
                button_hwnd,
                null_mut(),
                width - 80,
                height - 60,
                70,
                40,
                SWP_NOZORDER,
            );
            
            0
        }
        
        WM_CLOSE => {
            // Allow the window to be closed when explicitly asked
            println!("WM_CLOSE received, destroying window");
            
            // Clean up the state
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ChatMessages;
            if !state_ptr.is_null() {
                let _ = Box::from_raw(state_ptr);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            
            // Actually destroy the window
            DestroyWindow(hwnd);
            0
        }
        
        WM_DESTROY => {
            println!("WM_DESTROY received for window");
            
            // Clean up the state if not already cleaned up in WM_CLOSE
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ChatMessages;
            if !state_ptr.is_null() {
                println!("Cleaning up window user data in WM_DESTROY");
                let _ = Box::from_raw(state_ptr);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            
            // Don't post quit message - we want our UI thread to stay alive
            // PostQuitMessage(0);
            0
        },
        
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// Function to update the chat display
unsafe fn update_chat_display(hwnd: HWND, state: &ChatMessages) {
    let mut chat_text = String::new();
    for (sender, message) in &state.messages {
        chat_text.push_str(&format!("{}: {}\r\n", sender, message));
    }

    let chat_hwnd = GetDlgItem(hwnd, CHAT_HISTORY_ID);
    
    // Convert to wide string
    let chat_wstr: Vec<u16> = OsStr::new(&chat_text)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    
    SetWindowTextW(chat_hwnd, chat_wstr.as_ptr());
    
    // Scroll to bottom
    SendMessageW(chat_hwnd, EM_SETSEL as u32, 0, chat_text.len() as LPARAM);
    SendMessageW(chat_hwnd, EM_SETSEL as u32, chat_text.len() as WPARAM, chat_text.len() as LPARAM);
    SendMessageW(chat_hwnd, EM_SCROLLCARET as u32, 0, 0);
}

fn loword(l: u32) -> u16 {
    (l & 0xffff) as u16
}

fn hiword(l: u32) -> u16 {
    ((l >> 16) & 0xffff) as u16
}

// Debug-compatible Display for UiMessage
impl std::fmt::Debug for UiMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UiMessage::CreateWindow => write!(f, "CreateWindow"),
            UiMessage::AddMessage { sender, message } => 
                write!(f, "AddMessage {{ sender: {}, message: {} }}", sender, message),
            UiMessage::Close => write!(f, "Close"),
            UiMessage::EnsureWindowVisible => write!(f, "EnsureWindowVisible"),
            UiMessage::ForceForeground => write!(f, "ForceForeground"),
            UiMessage::ForceClose => write!(f, "ForceClose"),
        }
    }
}

// Callback for EnumChildWindows to close each child window
unsafe extern "system" fn enum_child_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
    println!("Destroying child window: {:?}", hwnd);
    DestroyWindow(hwnd);
    TRUE
}