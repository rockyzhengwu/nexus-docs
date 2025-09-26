const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("electronAPI", {
  openFile: () => ipcRenderer.invoke("dialog:openFile"),
});

const docAPI = require("doc-ai");
contextBridge.exposeInMainWorld("docAPI", docAPI);
