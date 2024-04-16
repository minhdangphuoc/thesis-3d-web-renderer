"use client";

import React from 'react';

export function TextFillInput ({ label, value, onChange, buttonText, onButtonClick }) {
  return (
    <div className="mb-4">
      <label className="block text-gray-700 text-sm font-bold mb-2">{label}</label>
      <div className="w-full flex">
        <input
            className="shadow appearance-none border w-full rounded py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
            type="text"
            value={value}
            onChange={onChange}
            />
        <button
          className="ml-4 bg-blue-500 hover:bg-blue-700 w-1/5 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
          type="button"
          onClick={onButtonClick}
          >
          {buttonText}
        </button>
      </div>
    </div>
  );
};