"use client";
import React, { useState, useEffect, useLayoutEffect } from "react";
import init, { start } from "../../../../engine/pkg/engine.js";
export function WasmCanvas() {
  const [loading, setLoading] = useState(false);
  const [started, setStarted] = useState(false);
  const [error, setError] = useState("");
  // const [wasm, setWasm] = useState();

  const loadExample = async () => {
    setLoading(true);
    init().then(()=> {
      start("https://raw.githubusercontent.com/minhdangphuoc/glTF-Sample-Models/main/2.0/SciFiHelmet/glTF/SciFiHelmet.gltf");
      setLoading(false);
      setStarted(true);
    }).catch((e)=>{
      console.log(e.message);
      if (
        !e.message.includes("Using exceptions for control flow,")
      ) {
        setError(`An error occurred loading": ${e}`);
        console.error(e);
        setStarted(false);
      } 

    })
  };

  useEffect(() => {
    if (started === false) loadExample();
  }, []);

  return (
    (started === true)?
    <div
      id="renderer-canvas"
      className="w-full	h-full block bg-black"
    ></div>:<></>
  );
}
