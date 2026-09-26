#include "interop.h"

#include <rhi/qrhi.h>

#include <QtCore/QByteArrayList>
#include <QtCore/QSize>
#include <QtGui/QVulkanInstance>
#include <QtGui/qtguiglobal.h>

#include <cstdio>

std::int32_t pictura_try_qrhi_import(std::uint64_t vkInstance,
                                     std::uint64_t physicalDevice,
                                     std::uint64_t device,
                                     std::uint32_t queueFamily,
                                     std::uint64_t image,
                                     std::uint32_t width,
                                     std::uint32_t height)
{
#if !QT_CONFIG(vulkan)
    static_cast<void>(vkInstance);
    static_cast<void>(physicalDevice);
    static_cast<void>(device);
    static_cast<void>(queueFamily);
    static_cast<void>(image);
    static_cast<void>(width);
    static_cast<void>(height);
    std::fprintf(stderr,
                 "pictura interop: this Qt build has no Vulkan feature; "
                 "--interop-probe is unavailable\n");
    return -1;
#else
    if (vkInstance == 0 || physicalDevice == 0 || device == 0) {
        std::fprintf(stderr, "pictura interop: missing Vulkan handles\n");
        return -1;
    }

    QVulkanInstance inst;
    inst.setVkInstance(reinterpret_cast<VkInstance>(vkInstance));
    if (!inst.create()) {
        std::fprintf(stderr, "pictura interop: QVulkanInstance::create() rejected the imported instance\n");
        return 0;
    }

    QRhiVulkanInitParams params;
    params.inst = &inst;
    const QByteArrayList required = QRhiVulkanInitParams::preferredExtensionsForImportedDevice();
    std::fprintf(stderr,
                 "pictura interop: device extensions QRhi needs: %s\n",
                 required.join(", ").constData());

    QRhiVulkanNativeHandles handles;
    handles.physDev = reinterpret_cast<VkPhysicalDevice>(physicalDevice);
    handles.dev = reinterpret_cast<VkDevice>(device);
    handles.gfxQueueFamilyIdx = queueFamily;
    handles.gfxQueueIdx = 0;

    QRhi* rhi = QRhi::create(QRhi::Vulkan, &params, {}, &handles);
    if (!rhi) {
        std::fprintf(stderr, "pictura interop: QRhi::create(importDevice) failed\n");
        return 0;
    }

    const auto* adopted = static_cast<const QRhiVulkanNativeHandles*>(rhi->nativeHandles());
    const bool sameDevice = adopted && adopted->dev == reinterpret_cast<VkDevice>(device);
    std::fprintf(stderr,
                 "pictura interop: QRhi backend=%s device=%s adopted_wgpu_device=%d queue_family=%u\n",
                 rhi->backendName(),
                 rhi->driverInfo().deviceName.constData(),
                 sameDevice ? 1 : 0,
                 queueFamily);

    bool wrapped = false;
    if (image != 0) {
        QRhiTexture* tex =
            rhi->newTexture(QRhiTexture::RGBA8, QSize(int(width), int(height)), 1, {});
        tex->setNativeLayout(int(VK_IMAGE_LAYOUT_GENERAL));
        QRhiTexture::NativeTexture native;
        native.object = image;
        native.layout = int(VK_IMAGE_LAYOUT_GENERAL);
        wrapped = tex->createFrom(native);
        std::fprintf(stderr,
                     "pictura interop: QRhiTexture::createFrom(wgpu VkImage)=%d (%ux%u)\n",
                     wrapped ? 1 : 0,
                     width,
                     height);
        delete tex;
    }

    delete rhi;
    return sameDevice ? 1 : 2;
#endif
}
