// Copyright (C) 2019 Florian Uekermann
// Copyright (C) 2010-2018 The RetroArch team
//
// ---------------------------------------------------------------------------------------------
// The following license statement only applies to this libretro API header (libretro_vulkan.h)
// ---------------------------------------------------------------------------------------------
//
// Permission is hereby granted, free of charge,
// to any person obtaining a copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software,
// and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
// WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

extern crate ash;
extern crate libc;

pub const HW_RENDER_INTERFACE_VERSION: libc::c_uint = 5;
pub const HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_VERSION: libc::c_uint = 1;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Image {
    pub image_view: ash::vk::ImageView,
    pub image_layout: ash::vk::ImageLayout,
    pub create_info: ash::vk::ImageViewCreateInfo,
}

pub type SetImageFn = unsafe extern "C" fn(
    handle: *mut libc::c_void,
    image: *const Image,
    num_semaphores: u32,
    semaphores: *const ash::vk::Semaphore,
    src_queue_family: u32,
);

pub type GetSyncIndexFn = unsafe extern "C" fn(handle: *mut libc::c_void) -> u32;
pub type GetSyncIndexMaskFn = unsafe extern "C" fn(handle: *mut libc::c_void) -> u32;
pub type SetCommandBuffersFn = unsafe extern "C" fn(
    handle: *mut libc::c_void,
    num_cmd: u32,
    cmd: *const ash::vk::CommandBuffer,
);
pub type WaitSyncIndexFn = unsafe extern "C" fn(handle: *mut libc::c_void);
pub type LockQueueFn = unsafe extern "C" fn(handle: *mut libc::c_void);
pub type UnlockQueueFn = unsafe extern "C" fn(handle: *mut libc::c_void);
pub type SetSignalSemaphoreFn =
    unsafe extern "C" fn(handle: *mut libc::c_void, semaphore: ash::vk::Semaphore);

pub type GetApplicationInfoFn = unsafe extern "C" fn() -> *const ash::vk::ApplicationInfo;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Context {
    pub gpu: ash::vk::PhysicalDevice,
    pub device: ash::vk::Device,
    pub queue: ash::vk::Queue,
    pub queue_family_index: u32,
    pub presentation_queue: ash::vk::Queue,
    pub presentation_queue_family_index: u32,
}

pub type CreateDeviceFn = unsafe extern "C" fn(
    context: *mut Context,
    instance: ash::vk::Instance,
    gpu: ash::vk::PhysicalDevice,
    surface: ash::vk::SurfaceKHR,
    get_instance_proc_addr: ash::vk::PFN_vkGetInstanceProcAddr,
    required_device_extensions: *const *const libc::c_char,
    num_required_device_extentions: libc::c_uint,
    required_device_layers: *const *const libc::c_char,
    num_required_device_layers: libc::c_uint,
    required_features: *const ash::vk::PhysicalDeviceFeatures,
) -> bool;

pub type DestroyDeviceFn = unsafe extern "C" fn();

/* Note on thread safety:
 * The Vulkan API is heavily designed around multi-threading, and
 * the libretro interface for it should also be threading friendly.
 * A core should be able to build command buffers and submit
 * command buffers to the GPU from any thread.
 */

#[derive(Clone, Debug)]
#[repr(C)]
pub struct HwRenderContextNegotiationInterface {
    /* Must be set to HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE. */
    pub retro_hw_render_context_negotiation_interface_type: libc::c_uint,
    /* Must be set to HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_VERSION. */
    pub interface_version: libc::c_uint,

    /* If non-NULL, returns a VkApplicationInfo struct that the frontend can use instead of
     * its "default" application info.
     */
    get_application_info: GetApplicationInfoFn,

    /* If non-NULL, the libretro core will choose one or more physical devices,
     * create one or more logical devices and create one or more queues.
     * The core must prepare a designated PhysicalDevice, Device, Queue and queue family index
     * which the frontend will use for its internal operation.
     *
     * If gpu is not VK_NULL_HANDLE, the physical device provided to the frontend must be this PhysicalDevice.
     * The core is still free to use other physical devices.
     *
     * The frontend will request certain extensions and layers for a device which is created.
     * The core must ensure that the queue and queue_family_index support GRAPHICS and COMPUTE.
     *
     * If surface is not VK_NULL_HANDLE, the core must consider presentation when creating the queues.
     * If presentation to "surface" is supported on the queue, presentation_queue must be equal to queue.
     * If not, a second queue must be provided in presentation_queue and presentation_queue_index.
     * If surface is not VK_NULL_HANDLE, the instance from frontend will have been created with supported for
     * VK_KHR_surface extension.
     *
     * The core is free to set its own queue priorities.
     * Device provided to frontend is owned by the frontend, but any additional device resources must be freed by core
     * in destroy_device callback.
     *
     * If this function returns true, a PhysicalDevice, Device and Queues are initialized.
     * If false, none of the above have been initialized and the frontend will attempt
     * to fallback to "default" device creation, as if this function was never called.
     */
    create_device: CreateDeviceFn,

    /* If non-NULL, this callback is called similar to context_destroy for HW_RENDER_INTERFACE.
     * However, it will be called even if context_reset was not called.
     * This can happen if the context never succeeds in being created.
     * destroy_device will always be called before the VkInstance
     * of the frontend is destroyed if create_device was called successfully so that the core has a chance of
     * tearing down its own device resources.
     *
     * Only auxillary resources should be freed here, i.e. resources which are not part of retro_vulkan_context.
     */
    destroy_device: DestroyDeviceFn,
}

#[derive(Clone)]
#[repr(C)]
pub struct HwRenderInterface {
    /* Must be set to HW_RENDER_INTERFACE. */
    pub interface_type: libc::c_uint,
    /* Must be set to HW_RENDER_INTERFACE_VERSION. */
    pub interface_version: libc::c_uint,

    /* Opaque handle to the Vulkan backend in the frontend
     * which must be passed along to all function pointers
     * in this interface.
     *
     * The rationale for including a handle here (which libretro v1
     * doesn't currently do in general) is:
     *
     * - Vulkan cores should be able to be freely threaded without lots of fuzz.
     *   This would break frontends which currently rely on TLS
     *   to deal with multiple cores loaded at the same time.
     * - Fixing this in general is TODO for an eventual libretro v2.
     */
    pub handle: *mut libc::c_void,

    /* The Vulkan instance the context is using. */
    pub instance: ash::vk::Instance,
    /* The physical device used. */
    pub gpu: ash::vk::PhysicalDevice,
    /* The logical device used. */
    pub device: ash::vk::Device,

    /* Allows a core to fetch all its needed symbols without having to link
     * against the loader itself. */
    pub get_device_proc_addr: ash::vk::PFN_vkGetDeviceProcAddr,
    pub get_instance_proc_addr: ash::vk::PFN_vkGetInstanceProcAddr,

    /* The queue the core must use to submit data.
     * This queue and index must remain constant throughout the lifetime
     * of the context.
     *
     * This queue will be the queue that supports graphics and compute
     * if the device supports compute.
     */
    pub queue: ash::vk::Queue,
    pub queue_index: libc::c_uint,

    /* Before calling retro_video_refresh_t with RETRO_HW_FRAME_BUFFER_VALID,
     * set which image to use for this frame.
     *
     * If num_semaphores is non-zero, the frontend will wait for the
     * semaphores provided to be signaled before using the results further
     * in the pipeline.
     *
     * Semaphores provided by a single call to set_image will only be
     * waited for once (waiting for a semaphore resets it).
     * E.g. set_image, video_refresh, and then another
     * video_refresh without set_image,
     * but same image will only wait for semaphores once.
     *
     * For this reason, ownership transfer will only occur if semaphores
     * are waited on for a particular frame in the frontend.
     *
     * Using semaphores is optional for synchronization purposes,
     * but if not using
     * semaphores, an image memory barrier in vkCmdPipelineBarrier
     * should be used in the graphics_queue.
     * Example:
     *
     * vkCmdPipelineBarrier(cmd,
     *    srcStageMask = VK_PIPELINE_STAGE_ALL_GRAPHICS_BIT,
     *    dstStageMask = VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT,
     *    image_memory_barrier = {
     *       srcAccessMask = VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
     *       dstAccessMask = VK_ACCESS_SHADER_READ_BIT,
     *    });
     *
     * The use of pipeline barriers instead of semaphores is encouraged
     * as it is simpler and more fine-grained. A layout transition
     * must generally happen anyways which requires a
     * pipeline barrier.
     *
     * The image passed to set_image must have imageUsage flags set to at least
     * VK_IMAGE_USAGE_TRANSFER_SRC_BIT and VK_IMAGE_USAGE_SAMPLED_BIT.
     * The core will naturally want to use flags such as
     * VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT and/or
     * VK_IMAGE_USAGE_TRANSFER_DST_BIT depending
     * on how the final image is created.
     *
     * The image must also have been created with MUTABLE_FORMAT bit set if
     * 8-bit formats are used, so that the frontend can reinterpret sRGB
     * formats as it sees fit.
     *
     * Images passed to set_image should be created with TILING_OPTIMAL.
     * The image layout should be transitioned to either
     * VK_IMAGE_LAYOUT_GENERIC or VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL.
     * The actual image layout used must be set in image_layout.
     *
     * The image must be a 2D texture which may or not be layered
     * and/or mipmapped.
     *
     * The image must be suitable for linear sampling.
     * While the image_view is typically the only field used,
     * the frontend may want to reinterpret the texture as sRGB vs.
     * non-sRGB for example so the VkImageViewCreateInfo used to
     * create the image view must also be passed in.
     *
     * The data in the pointer to the image struct will not be copied
     * as the pNext field in create_info cannot be reliably deep-copied.
     * The image pointer passed to set_image must be valid until
     * retro_video_refresh_t has returned.
     *
     * If frame duping is used when passing NULL to retro_video_refresh_t,
     * the frontend is free to either use the latest image passed to
     * set_image or reuse the older pointer passed to set_image the
     * frame RETRO_HW_FRAME_BUFFER_VALID was last used.
     *
     * Essentially, the lifetime of the pointer passed to
     * retro_video_refresh_t should be extended if frame duping is used
     * so that the frontend can reuse the older pointer.
     *
     * The image itself however, must not be touched by the core until
     * wait_sync_index has been completed later. The frontend may perform
     * layout transitions on the image, so even read-only access is not defined.
     * The exception to read-only rule is if GENERAL layout is used for the image.
     * In this case, the frontend is not allowed to perform any layout transitions,
     * so concurrent reads from core and frontend are allowed.
     *
     * If frame duping is used, or if set_command_buffers is used,
     * the frontend will not wait for any semaphores.
     *
     * The src_queue_family is used to specify which queue family
     * the image is currently owned by. If using multiple queue families
     * (e.g. async compute), the frontend will need to acquire ownership of the
     * image before rendering with it and release the image afterwards.
     *
     * If src_queue_family is equal to the queue family (queue_index),
     * no ownership transfer will occur.
     * Similarly, if src_queue_family is VK_QUEUE_FAMILY_IGNORED,
     * no ownership transfer will occur.
     *
     * The frontend will always release ownership back to src_queue_family.
     * Waiting for frontend to complete with wait_sync_index() ensures that
     * the frontend has released ownership back to the application.
     * Note that in Vulkan, transfering ownership is a two-part process.
     *
     * Example frame:
     *  - core releases ownership from src_queue_index to queue_index with VkImageMemoryBarrier.
     *  - core calls set_image with src_queue_index.
     *  - Frontend will acquire the image with src_queue_index -> queue_index as well, completing the ownership transfer.
     *  - Frontend renders the frame.
     *  - Frontend releases ownership with queue_index -> src_queue_index.
     *  - Next time image is used, core must acquire ownership from queue_index ...
     *
     * Since the frontend releases ownership, we cannot necessarily dupe the frame because
     * the core needs to make the roundtrip of ownership transfer.
     */
    pub set_image: SetImageFn,

    /* Get the current sync index for this frame which is obtained in
     * frontend by calling e.g. vkAcquireNextImageKHR before calling
     * retro_run().
     *
     * This index will correspond to which swapchain buffer is currently
     * the active one.
     *
     * Knowing this index is very useful for maintaining safe asynchronous CPU
     * and GPU operation without stalling.
     *
     * The common pattern for synchronization is to receive fences when
     * submitting command buffers to Vulkan (vkQueueSubmit) and add this fence
     * to a list of fences for frame number get_sync_index().
     *
     * Next time we receive the same get_sync_index(), we can wait for the
     * fences from before, which will usually return immediately as the
     * frontend will generally also avoid letting the GPU run ahead too much.
     *
     * After the fence has signaled, we know that the GPU has completed all
     * GPU work related to work submitted in the frame we last saw get_sync_index().
     *
     * This means we can safely reuse or free resources allocated in this frame.
     *
     * In theory, even if we wait for the fences correctly, it is not technically
     * safe to write to the image we earlier passed to the frontend since we're
     * not waiting for the frontend GPU jobs to complete.
     *
     * The frontend will guarantee that the appropriate pipeline barrier
     * in graphics_queue has been used such that
     * VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT cannot
     * start until the frontend is done with the image.
     */
    pub get_sync_index: GetSyncIndexFn,

    /* Returns a bitmask of how many swapchain images we currently have
     * in the frontend.
     *
     * If bit #N is set in the return value, get_sync_index can return N.
     * Knowing this value is useful for preallocating per-frame management
     * structures ahead of time.
     *
     * While this value will typically remain constant throughout the
     * applications lifecycle, it may for example change if the frontend
     * suddently changes fullscreen state and/or latency.
     *
     * If this value ever changes, it is safe to assume that the device
     * is completely idle and all synchronization objects can be deleted
     * right away as desired.
     */
    pub get_sync_index_mask: GetSyncIndexMaskFn,

    /* Instead of submitting the command buffer to the queue first, the core
     * can pass along its command buffer to the frontend, and the frontend
     * will submit the command buffer together with the frontends command buffers.
     *
     * This has the advantage that the overhead of vkQueueSubmit can be
     * amortized into a single call. For this mode, semaphores in set_image
     * will be ignored, so vkCmdPipelineBarrier must be used to synchronize
     * the core and frontend.
     *
     * The command buffers in set_command_buffers are only executed once,
     * even if frame duping is used.
     *
     * If frame duping is used, set_image should be used for the frames
     * which should be duped instead.
     *
     * Command buffers passed to the frontend with set_command_buffers
     * must not actually be submitted to the GPU until retro_video_refresh_t
     * is called.
     *
     * The frontend must submit the command buffer before submitting any
     * other command buffers provided by set_command_buffers. */
    pub set_command_buffers: SetCommandBuffersFn,

    /* Waits on CPU for device activity for the current sync index to complete.
     * This is useful since the core will not have a relevant fence to sync with
     * when the frontend is submitting the command buffers. */
    pub wait_sync_index: WaitSyncIndexFn,

    /* If the core submits command buffers itself to any of the queues provided
     * in this interface, the core must lock and unlock the frontend from
     * racing on the VkQueue.
     *
     * Queue submission can happen on any thread.
     * Even if queue submission happens on the same thread as retro_run(),
     * the lock/unlock functions must still be called.
     *
     * NOTE: Queue submissions are heavy-weight. */
    pub lock_queue: LockQueueFn,
    pub unlock_queue: UnlockQueueFn,

    /* Sets a semaphore which is signaled when the image in set_image can safely be reused.
     * The semaphore is consumed next call to retro_video_refresh_t.
     * The semaphore will be signalled even for duped frames.
     * The semaphore will be signalled only once, so set_signal_semaphore should be called every frame.
     * The semaphore may be VK_NULL_HANDLE, which disables semaphore signalling for next call to retro_video_refresh_t.
     *
     * This is mostly useful to support use cases where you're rendering to a single image that
     * is recycled in a ping-pong fashion with the frontend to save memory (but potentially less throughput).
     */
    pub set_signal_semaphore: SetSignalSemaphoreFn,
}
