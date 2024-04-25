# thesis-3d-web-renderer

![screenshot](screenshots/screenshot.png)

## Prerequisites

### Dependences

``` shell
# Install Rust https://www.rust-lang.org/tools/install
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
# Install NodeJS https://nodejs.org/en/download
```

### Browsers

- Details of current-supported WebGPU Browsers
1. [caniuse.com/webgpu](https://caniuse.com/webgpu)
1. [developer.mozilla.org/en-US/docs/Web/API/WebGPU_API](https://developer.mozilla.org/en-US/docs/Web/API/WebGPU_API)

## Build & Run

``` shell
# 1. Go to web example directory
cd demo

# 2. Install package
npm install

# 3. Build
npm run build

# 3. Run
npm run start

# 4. [Optional] Dev
npm run dev

```