use fj_interop::ext::ArrayExt;
use fj_math::Transform;

use crate::{
    objects::Objects,
    partial::{MaybePartial, PartialGlobalEdge, PartialHalfEdge},
    validate::ValidationError,
};

use super::TransformObject;

impl TransformObject for PartialHalfEdge {
    fn transform(
        self,
        transform: &Transform,
        objects: &Objects,
    ) -> Result<Self, ValidationError> {
        let curve: MaybePartial<_> = self
            .curve()
            .into_partial()
            .transform(transform, objects)?
            .into();
        let vertices = self.vertices().try_map_ext(
            |vertex| -> Result<_, ValidationError> {
                let mut vertex =
                    vertex.into_partial().transform(transform, objects)?;
                vertex.curve = curve.clone();
                Ok(vertex)
            },
        )?;
        let mut global_form = self
            .global_form()
            .into_partial()
            .transform(transform, objects)?;
        if let Some(curve) = curve.global_form() {
            global_form.curve = curve;
        }

        Ok(Self::default()
            .with_curve(curve)
            .with_vertices(vertices)
            .with_global_form(global_form))
    }
}

impl TransformObject for PartialGlobalEdge {
    fn transform(
        self,
        transform: &Transform,
        objects: &Objects,
    ) -> Result<Self, ValidationError> {
        let curve = self.curve.transform(transform, objects)?;
        let vertices = self
            .vertices
            .map(|vertices| {
                vertices.try_map_ext(|vertex| -> Result<_, ValidationError> {
                    vertex.transform(transform, objects)
                })
            })
            .transpose()?;

        Ok(Self { curve, vertices })
    }
}
