import { ocrResultAtom } from "@/state/ai";
import { useAtom } from "jotai";

export default function Editor() {
  const [ocrResult, setOcrResult] = useAtom(ocrResultAtom);
  const content = ocrResult.map((item) => <li key={item}>{item.content}</li>);

  return (
    <>
      <h1>Editor </h1>
      {content}
    </>
  );
}
