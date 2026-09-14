#pragma once

#include <cstdint>

// Attempt to adopt a wgpu-created Vulkan instance/device into Qt's RHI, then
// wrap the wgpu offscreen VkImage as a QRhiTexture.
// Returns 1 when the imported device is the same VkDevice, 0 when QRhi cannot
// be created, -1 on bad input.
std::int32_t pictura_try_qrhi_import(std::uint64_t vkInstance,
                                     std::uint64_t physicalDevice,
                                     std::uint64_t device,
                                     std::uint32_t queueFamily,
                                     std::uint64_t image,
                                     std::uint32_t width,
                                     std::uint32_t height);
