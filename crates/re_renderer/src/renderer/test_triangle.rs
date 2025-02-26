use smallvec::smallvec;

use crate::{
    context::SharedRendererData,
    include_file,
    view_builder::ViewBuilder,
    wgpu_resources::{
        GpuRenderPipelineHandle, PipelineLayoutDesc, RenderPipelineDesc, ShaderModuleDesc,
    },
};

use super::*;

pub struct TestTriangle {
    render_pipeline: GpuRenderPipelineHandle,
}

#[derive(Clone)]
pub struct TestTriangleDrawData;

impl DrawData for TestTriangleDrawData {
    type Renderer = TestTriangle;
}

impl TestTriangleDrawData {
    pub fn new(ctx: &mut RenderContext) -> Self {
        ctx.renderers.get_or_create::<_, TestTriangle>(
            &ctx.shared_renderer_data,
            &mut ctx.gpu_resources,
            &ctx.device,
            &mut ctx.resolver,
        );

        TestTriangleDrawData {}
    }
}

impl Renderer for TestTriangle {
    type RendererDrawData = TestTriangleDrawData;

    fn create_renderer<Fs: FileSystem>(
        shared_data: &SharedRendererData,
        pools: &mut WgpuResourcePools,
        device: &wgpu::Device,
        resolver: &mut FileResolver<Fs>,
    ) -> Self {
        let render_pipeline = pools.render_pipelines.get_or_create(
            device,
            &RenderPipelineDesc {
                label: "Test Triangle".into(),
                pipeline_layout: pools.pipeline_layouts.get_or_create(
                    device,
                    &PipelineLayoutDesc {
                        label: "global only".into(),
                        entries: vec![shared_data.global_bindings.layout],
                    },
                    &pools.bind_group_layouts,
                ),
                vertex_entrypoint: "vs_main".into(),
                vertex_handle: pools.shader_modules.get_or_create(
                    device,
                    resolver,
                    &ShaderModuleDesc {
                        label: "test_triangle (vertex)".into(),
                        source: include_file!("../../shader/test_triangle.wgsl"),
                    },
                ),
                fragment_entrypoint: "fs_main".into(),
                fragment_handle: pools.shader_modules.get_or_create(
                    device,
                    resolver,
                    &ShaderModuleDesc {
                        label: "test_triangle (fragment)".into(),
                        source: include_file!("../../shader/test_triangle.wgsl"),
                    },
                ),
                vertex_buffers: smallvec![],
                render_targets: smallvec![Some(ViewBuilder::MAIN_TARGET_COLOR_FORMAT.into())],
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: ViewBuilder::MAIN_TARGET_DEPTH_FORMAT,
                    depth_compare: wgpu::CompareFunction::Always,
                    depth_write_enabled: true, // writes some depth for testing
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: ViewBuilder::MAIN_TARGET_DEFAULT_MSAA_STATE,
            },
            &pools.pipeline_layouts,
            &pools.shader_modules,
        );

        TestTriangle { render_pipeline }
    }

    fn draw<'a>(
        &self,
        pools: &'a WgpuResourcePools,
        pass: &mut wgpu::RenderPass<'a>,
        _draw_data: &TestTriangleDrawData,
    ) -> anyhow::Result<()> {
        let pipeline = pools.render_pipelines.get_resource(self.render_pipeline)?;
        pass.set_pipeline(pipeline);
        pass.draw(0..3, 0..1);
        Ok(())
    }
}
