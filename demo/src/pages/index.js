import { useEffect, useState } from "react";
import { Inter } from "next/font/google";
import { WasmCanvas } from "../components/WasmCanvas/WasmCanvas.js"
import { TextFillInput } from "../components/TextInput/TextInput.js"
const inter = Inter({ subsets: ["latin"] });

// Print some useful link
console.log("Useful links:");
console.log("ScullCup: https://raw.githubusercontent.com/minhdangphuoc/model/main/ScullCup/scene.gltf");
console.log("DamagedHelmet: https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Models/main/2.0/DamagedHelmet/glTF/DamagedHelmet.gltf");
console.log("SciFiHelmet: https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Models/main/2.0/SciFiHelmet/glTF/SciFiHelmet.gltf");
console.log("Avocado: https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Models/main/2.0/Avocado/glTF/Avocado.gltf");
console.log("Duck: https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Models/main/2.0/Duck/glTF/Duck.gltf");

export default function Home() {
  const [name, setName] = useState('');
  const [clicked, setButtonClick] = useState(false);
  const [started, setStarted] = useState(false);

  const handleNameChange = (e) => {
    setName(e.target.value);
  };

  const handleClickChange = () => {
    setButtonClick(true);
  };

  useEffect(()=>{
    if (clicked === true) {
      if (started === true) {
        window.location.reload();
      }
    }
  })

  return (
    <main
      className={`flex min-h-screen flex-col items-center justify-between ${inter.className}`}
    >
      <div className="h-screen w-screen"> 
        <div className="fixed bottom-0 w-fullfixed left-1/2 transform -translate-x-1/2 w-1/2">
          <TextFillInput label="URL" value={name} onChange={handleNameChange} onButtonClick={handleClickChange} buttonText={started?  "Reload" : "Run" }/>
        </div>
        <WasmCanvas url={name} onClick={clicked} setButtonClick={setButtonClick} started={started} setStarted={setStarted}/>
      </div>
    </main>
  );
}
