use std::marker::PhantomData;

use smithay::backend::renderer::Renderer;

use crate::surface::{
    ArrangeContext, RenderTransformContext, SurfaceLogicalRectangle, SurfaceLogicalSize,
    SurfaceTransformer, WindowTransform,
};

pub struct Unarranged;
pub struct Arranged;
pub struct PreRendered;
pub struct PostRendered;

pub struct SurfaceTransformPipeline<R>
where
    R: Renderer,
{
    transformers: Vec<Box<dyn SurfaceTransformer<R>>>,
}

impl<R> SurfaceTransformPipeline<R>
where
    R: Renderer,
{
    pub fn new() -> Self {
        Self {
            transformers: Vec::new(),
        }
    }

    pub fn push<T>(&mut self, transformer: T)
    where
        T: SurfaceTransformer<R> + 'static,
    {
        self.transformers.push(Box::new(transformer));
    }

    pub fn begin<T>(
        &self,
        outer: T,
        arrange_ctx: ArrangeContext,
    ) -> SurfaceTransformRun<'_, R, Unarranged>
    where
        T: Into<WindowTransform>,
    {
        SurfaceTransformRun {
            pipeline: self,
            transform: outer.into(),
            arrange_ctx,
            _phase: PhantomData,
        }
    }
}

impl<R> Default for SurfaceTransformPipeline<R>
where
    R: Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

pub struct SurfaceTransformRun<'a, R, Phase>
where
    R: Renderer,
{
    pipeline: &'a SurfaceTransformPipeline<R>,
    transform: WindowTransform,
    arrange_ctx: ArrangeContext,
    _phase: PhantomData<Phase>,
}

impl<'a, R> SurfaceTransformRun<'a, R, Unarranged>
where
    R: Renderer,
{
    pub fn arrange(self) -> SurfaceTransformRun<'a, R, Arranged> {
        let transform = self
            .pipeline
            .transformers
            .iter()
            .fold(self.transform, |transform, transformer| {
                transformer.arrange(transform, &self.arrange_ctx)
            });

        SurfaceTransformRun {
            pipeline: self.pipeline,
            transform,
            arrange_ctx: self.arrange_ctx,
            _phase: PhantomData,
        }
    }
}

impl<'a, R> SurfaceTransformRun<'a, R, Arranged>
where
    R: Renderer,
{
    pub fn transform(&self) -> WindowTransform {
        self.transform
    }

    pub fn configure_size(&self) -> SurfaceLogicalSize {
        self.transform.configure_size
    }

    pub fn client_rect(&self) -> SurfaceLogicalRectangle {
        self.transform.client_rect
    }

    pub fn render_pre(
        self,
        ctx: &mut RenderTransformContext<'_, R>,
    ) -> Result<SurfaceTransformRun<'a, R, PreRendered>, R::Error> {
        ctx.transform = self.transform;

        for transformer in &self.pipeline.transformers {
            transformer.render_pre(ctx)?;
        }

        Ok(SurfaceTransformRun {
            pipeline: self.pipeline,
            transform: self.transform,
            arrange_ctx: self.arrange_ctx,
            _phase: PhantomData,
        })
    }
}

impl<'a, R> SurfaceTransformRun<'a, R, PreRendered>
where
    R: Renderer,
{
    pub fn transform(&self) -> WindowTransform {
        self.transform
    }

    pub fn client_rect(&self) -> SurfaceLogicalRectangle {
        self.transform.client_rect
    }

    pub fn render_post(
        self,
        ctx: &mut RenderTransformContext<'_, R>,
    ) -> Result<SurfaceTransformRun<'a, R, PostRendered>, R::Error> {
        ctx.transform = self.transform;

        for transformer in self.pipeline.transformers.iter().rev() {
            transformer.render_post(ctx)?;
        }

        Ok(SurfaceTransformRun {
            pipeline: self.pipeline,
            transform: self.transform,
            arrange_ctx: self.arrange_ctx,
            _phase: PhantomData,
        })
    }
}

impl<'a, R> SurfaceTransformRun<'a, R, PostRendered>
where
    R: Renderer,
{
    pub fn transform(&self) -> WindowTransform {
        self.transform
    }
}
