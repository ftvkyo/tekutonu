# tekutonu

## Prerequisites

Install build dependencies:
- cmake, make
- pkg-config
- fontconfig library

Install runtime dependencies:
- vulkan stuff

### Running under WSL with Nvidia GPU

You will need recent Nvidia drivers on the host along with Dozen - interface from Vulkan to D3D12 in Linux, [implemented in Mesa][dzn].

On [ArchWSL], install `mesa-d3d12` from AUR.
Additionally, you'll need to install `libxkbcommon` because of a problem in `winit`.

[dzn]: https://www.phoronix.com/news/Vulkan-On-Direct3D-12-Dzn-Merge
[ArchWSL]: https://github.com/yuk7/ArchWSL
