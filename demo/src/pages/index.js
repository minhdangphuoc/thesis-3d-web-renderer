import Image from "next/image";
import { Inter } from "next/font/google";
import { WasmCanvas } from "./component/WasmCanvas"
const inter = Inter({ subsets: ["latin"] });

export default function Home() {
  return (
    <main
      className={`flex min-h-screen flex-col items-center justify-between ${inter.className}`}
    >
      <div className="">
        <WasmCanvas/>
      </div>
    </main>
  );
}
