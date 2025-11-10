import { atom } from "jotai";

export const ocrResultAtom = atom([]);
export const markdownContentAtom = atom(
  "# Hi, *Pluto*! \n This is **not** a paragraph.",
);
