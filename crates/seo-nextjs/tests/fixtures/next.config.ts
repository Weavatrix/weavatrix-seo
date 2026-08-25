const nextConfig = {
  basePath: '/app',
  trailingSlash: true,
  async redirects() {
    return [{ source: '/old', destination: '/new', permanent: true }];
  },
};

export default nextConfig;
