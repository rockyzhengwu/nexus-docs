import { useAtom } from "jotai";
import { imgUrlAtom, pdfUrlAtom } from "@/state/input";
import PdfViewer from "@/components/pdf-viewer";
import ImageViewer from "@/components/img-viewer";
import Editor from "@/components/editor";
import { openViewerAtom } from "@/state/home";
import clsx from "clsx";

export default function Home() {
  const [imgUrl, setImgUrl] = useAtom(imgUrlAtom);
  const [pdfUrl, setPdfUrl] = useAtom(pdfUrlAtom);
  const [openViewer, setOpenViewer] = useAtom(openViewerAtom);

  return (
    <>
      <div className="flex h-screen space-x-4">
        <div
          className={clsx(
            "w-1/2 bg-white p-8 overflow-y-auto border-solid border-2 ",
            { hidden: !openViewer },
          )}
        >
          {pdfUrl ? (
            <PdfViewer pdfUrl={pdfUrl} />
          ) : imgUrl ? (
            <ImageViewer imgUrl={imgUrl} />
          ) : null}
        </div>

        <div
          className={clsx(
            " bg-white  p-8 overflow-y-auto border-solid boarder-2",
            { "w-1/2": openViewer, "w-full": !openViewer },
          )}
        >
          <Editor />
        </div>
      </div>
    </>
  );
}
