import { ocrResultAtom, markdownContentAtom } from "@/state/ai";
import { useAtom } from "jotai";
import { useRef, useEffect, useState } from "react";

import "cherry-markdown/dist/cherry-markdown.css";
import Cherry from "cherry-markdown";

export default function Editor() {
  const [ocrResult, setOcrResult] = useAtom(ocrResultAtom);
  const [markdownContent, setMarkdownContent] = useAtom(markdownContentAtom);
  const editorRef = useRef(null);
  const [editor, setEditor] = useState(null);
  const isInitialized = useRef(false);

  useEffect(() => {
    if (editorRef.current === null || isInitialized.current) return;
    try {
      // 初始化编辑器
      const config = {
        el: editorRef.current,
        value: markdownContent,
        callback: {
          afterChange: () => console.log("change"),
        },
      };

      const cherry = new Cherry(config);
      setEditor(cherry);
      isInitialized.current = true;
    } catch (error) {
      console.error("初始化 Cherry 编辑器失败", error);
    }
    return () => {
      if (editor) {
        editor.destroy();
      }
    };
  }, [editor]);

  useEffect(() => {
    if (editor !== null) {
      editor.setMarkdown(markdownContent);
    }
  }, [markdownContent]);

  return (
    <>
      <div ref={editorRef} id="markdown-container" />
    </>
  );
}
