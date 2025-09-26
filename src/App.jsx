import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import AppSidebar from "@/components/sidebar";
import { Outlet } from "react-router-dom";

const App = () => {
  return (
    <>
      <SidebarProvider>
        <AppSidebar />
        <main className="w-full bg-gray-100">
          <SidebarTrigger />
          <Outlet />
        </main>
      </SidebarProvider>
      ;
    </>
  );
};

export default App;
