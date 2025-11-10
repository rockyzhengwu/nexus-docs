import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenuItem,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenuButton,
  SidebarGroupContent,
} from "@/components/ui/sidebar";

import { Plus } from "lucide-react";
import { useAtom } from "jotai";
import { pdfAtom, imgAtom, pdfUrlAtom, imgUrlAtom } from "@/state/input";
import { ocrResultAtom, markdownContentAtom } from "@/state/ai";

export default function AppSidebar() {
  const [pdf, setPdf] = useAtom(pdfAtom);
  const [img, setImg] = useAtom(imgAtom);
  const [pdfUrl, setPdfUrl] = useAtom(pdfUrlAtom);
  const [imgUrl, setImgUrl] = useAtom(imgUrlAtom);
  const [ocrResult, setOcrResult] = useAtom(ocrResultAtom);
  const [markdownContent, setMarkdownContent] = useAtom(markdownContentAtom);

  function getFileExtension(filename) {
    const parts = filename.split(".");
    if (parts.length > 1) {
      return parts.pop(); // Returns the last part, which is the extension
    }
    return ""; // No extension found
  }

  function base64ToArrayBuffer(base64) {
    const binaryString = atob(base64);

    const length = binaryString.length;
    const bytes = new Uint8Array(length);

    for (let i = 0; i < length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }

    return bytes;
  }

  const handleSelectFile = async () => {
    const url = await window.electronAPI.openFile();
    if (url && url.startsWith("data:image")) {
      setImgUrl(url);
      const base64DataOnly2 = url.replace(/^data:[^;]+;base64,/, "");
      const buffer = base64ToArrayBuffer(base64DataOnly2);
      console.log("start doc api markdwon:", buffer);
      const markdown = await window.docAPI.markdown(buffer);
      setMarkdownContent(markdown);
      console.log(markdown);
    } else if (url) {
      setPdfUrl(url);
    } else {
      console.log("url is None");
    }
  };

  return (
    <Sidebar>
      <SidebarHeader />
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Application</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenuItem>
              <SidebarMenuButton asChild onClick={handleSelectFile}>
                <a href="#">
                  <Plus />
                  <span>Iput Flie</span>
                </a>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter />
    </Sidebar>
  );
}
