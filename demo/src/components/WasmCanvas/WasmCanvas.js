import React, { useState, useEffect, useLayoutEffect } from "react";
import init, { start, stop } from "../../../../engine/pkg/engine";
import { Router } from "next/router";

export function WasmCanvas({url, onClick, setButtonClick, started, setStarted}) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  //const [started, setStarted] = useState(false);

  // const [wasm, setWasm] = useState();

  const loadExample = (url) => {
    setLoading(true);
    const startInit = async() => {await init().then(()=> {
      start(url);
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

    })}
    startInit();
  };

  useEffect(() => {
    if (onClick === true) {
      console.log("Clicked");
      loadExample(url);
      setButtonClick(false);
    }
  }, [onClick]);

  return (
    <div
      id="renderer-canvas"
      className="w-full	h-full block bg-black"
    ></div>
  );
}
