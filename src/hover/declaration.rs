//! Hover at a declaration site: the same signature and documentation a
//! reference to the symbol would show.
//!
//! Clients that navigate through the protocol rather than a screen land on
//! declarations (go-to-definition, document symbols, references all answer
//! with declaration positions) and need the resolved signature, inherited
//! docblock and inferred types without reading the file themselves.

use tower_lsp_server::gen_lsp_types::Hover;

use crate::Backend;
use crate::class_lookup::find_documented_class_at_offset;
use crate::references::MemberRefKind;

use super::HoverSite;
use super::member::HoverMemberHit;

impl Backend {
    /// Hover for a class, interface, trait or enum name at its declaration.
    pub(super) fn hover_class_declaration(&self, site: &HoverSite) -> Option<Hover> {
        let class = find_documented_class_at_offset(&site.ctx.classes, site.offset)?;
        Some(self.hover_for_class_info(class, site.uri, site.content))
    }

    /// Hover for a method, property, constant or enum case name at its
    /// declaration, rendered from the fully resolved class so inherited
    /// documentation and virtual members are included.
    pub(super) fn hover_member_declaration(
        &self,
        site: &HoverSite,
        name: &str,
        is_method: bool,
    ) -> Option<Hover> {
        let class = find_documented_class_at_offset(&site.ctx.classes, site.offset)?;
        let resolved_class_loader = site.loaders.resolved_class_loader();
        let merged = crate::virtual_members::resolve_class_fully_cached(
            class,
            resolved_class_loader,
            &self.resolved_class_cache,
        );
        let kind = if is_method {
            MemberRefKind::Method
        } else {
            MemberRefKind::Either
        };
        Some(match Self::find_member_for_hover(&merged, name, kind)? {
            HoverMemberHit::Method(m) => {
                self.hover_for_method(&m, &merged, resolved_class_loader, site.uri, site.content)
            }
            HoverMemberHit::Property(p) => {
                self.hover_for_property(&p, &merged, resolved_class_loader)
            }
            HoverMemberHit::Constant(c) => {
                self.hover_for_constant(&c, &merged, resolved_class_loader)
            }
        })
    }
}
