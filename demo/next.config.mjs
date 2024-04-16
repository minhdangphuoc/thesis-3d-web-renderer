/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  webpack: function (config, options) {
    config.experiments = { asyncWebAssembly: true, topLevelAwait: true, syncWebAssembly: true, layers: true};
    return config;
  },
};

export default nextConfig;
