import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import AppSidebar from "@/components/sidebar";
import { Outlet } from "react-router-dom";
import { ArrowLeft } from "lucide-react";
import { openViewerAtom } from "@/state/home";
import { useAtom } from "jotai";

const App = () => {
  const [openViewer, setOpenViewer] = useAtom(openViewerAtom);

  const toggleViewser = () => {
    setOpenViewer(!openViewer);
  };

  return (
    <>
      <SidebarProvider>
        <AppSidebar />
        <main className="w-full bg-gray-100">
          <div className="flex  flex-shrink-0 ">
            <SidebarTrigger />
            <button className="flex " onClick={toggleViewser}>
              <ArrowLeft />
            </button>
          </div>
          <Outlet />
        </main>
      </SidebarProvider>
      ;
    </>
  );
};

export default App;
