import { useAtom } from "jotai";
import { imgUrlAtom, pdfUrlAtom } from "@/state/input";
import PdfViewer from "@/components/pdf-viewer";
import ImageViewer from "@/components/img-viewer";
import Editor from "@/components/editor";

export default function Home() {
  const [imgUrl, setImgUrl] = useAtom(imgUrlAtom);
  const [pdfUrl, setPdfUrl] = useAtom(pdfUrlAtom);

  return (
    <>
      <div className="flex h-screen space-x-4">
        <div className="w-1/2 bg-white p-8 overflow-y-auto border-solid border-2">
          {pdfUrl ? (
            <PdfViewer pdfUrl={pdfUrl} />
          ) : imgUrl ? (
            <ImageViewer imgUrl={imgUrl} />
          ) : null}
        </div>

        <div className="w-1/2 bg-white  p-8 overflow-y-auto border-solid boarder-2">
          <Editor />
        </div>
      </div>
    </>
  );
}
