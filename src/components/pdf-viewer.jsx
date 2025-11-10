import { useState, useRef, useEffect } from "react";
import clsx from "clsx";

import { pdfjs, Document, Page } from "react-pdf";

import "react-pdf/dist/Page/AnnotationLayer.css";
import "react-pdf/dist/Page/TextLayer.css";

pdfjs.GlobalWorkerOptions.workerSrc = new URL(
  "pdfjs-dist/build/pdf.worker.min.mjs",
  import.meta.url,
).toString();

const options = {
  cMapUrl: "/cmaps/",
  standardFontDataUrl: "/standard_fonts/",
  wasmUrl: "/wasm/",
};

export default function PdfViewer({ pdfUrl }) {
  const [numPages, setNumPages] = useState(0);
  const [pageNumber, setPageNumber] = useState(1);

  const nextPage = () => {
    if (pageNumber >= numPages) {
      return;
    } else {
      setPageNumber(pageNumber + 1);
    }
  };

  const prePage = () => {
    if (pageNumber === 1) {
      return;
    }
    setPageNumber(pageNumber - 1);
  };

  const onDocumentLoadSuccess = ({ numPages }) => {
    setPageNumber(1);
    setNumPages(numPages);
  };

  return (
    <>
      <div>
        <Document
          file={pdfUrl}
          onLoadSuccess={onDocumentLoadSuccess}
          options={options}
        >
          <Page pageNumber={pageNumber} size="A4" width={800} />
        </Document>
        <div className="flex justify-between items-center mt-4 w-full max-w-4xl">
          <button
            id="prev-page-btn"
            className={clsx(
              "px-4 py-2 font-semibold text-sm bg-indigo-600 text-white rounded-md shadow-sm   transition-colors duration-200",
              pageNumber === 1 && "cursor-not-allowed opacity-50",
              pageNumber !== 1 && "hover:bg-indigo-700",
            )}
            onClick={prePage}
            disabled={pageNumber === 1}
          >
            Previous Page
          </button>
          <div id="page-info" className="text-sm font-medium text-gray-700">
            Page <span id="current-page">{pageNumber}</span> of{" "}
            <span id="total-pages">{numPages}</span>
          </div>
          <button
            id="next-page-btn"
            className={clsx(
              "px-4 py-2 font-semibold text-sm bg-indigo-600 text-white rounded-md shadow-sm transition-colors duration-200 ",
              pageNumber === numPages &&
                "cursor-not-allowed opacity-50 hover:none",
              pageNumber !== numPages && "hover:bg-indigo-700",
            )}
            onClick={nextPage}
            disabled={pageNumber === numPages}
          >
            Next Page
          </button>
        </div>
      </div>
    </>
  );
}
