/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  webpack: function (config, options) {
    config.experiments = { asyncWebAssembly: true, topLevelAwait: true, syncWebAssembly: true, layers: true};
    return config;
  },
};

module.exports = {
  pageExtensions: ['page.tsx', 'page.ts', 'page.jsx', 'page.js']
}

export default nextConfig;
