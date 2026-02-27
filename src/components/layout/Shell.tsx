import { Outlet } from "react-router-dom";
import { Sidebar } from "./Sidebar";
import { Header } from "./Header";

export function Shell() {
  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar />
      <div className="flex-1 overflow-y-auto bg-[#FAFAF9]">
        <Header />
        <main className="mx-auto max-w-[1200px] px-8 py-8">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
