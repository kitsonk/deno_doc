use super::partition::Partition;
use super::DocNodeKindCtx;
use super::DocNodeWithContext;
use super::GenerateCtx;
use super::ShortPath;
use super::UrlResolveKind;
use deno_ast::ModuleSpecifier;
use indexmap::IndexMap;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
struct SidepanelPartitionSymbolCtx {
  kind: Vec<DocNodeKindCtx>,
  name: String,
  href: String,
  active: bool,
  deprecated: bool,
}

impl SidepanelPartitionSymbolCtx {
  fn new(
    nodes: &[&DocNodeWithContext],
    active: bool,
    href: String,
    name: String,
  ) -> Self {
    Self {
      kind: nodes
        .iter()
        .map(|node| node.kind_with_drilldown.into())
        .collect(),
      name,
      href,
      active,
      deprecated: super::util::all_deprecated(nodes),
    }
  }
}

#[derive(Debug, Serialize, Clone)]
struct SidepanelPartitionCtx {
  name: String,
  symbols: Vec<SidepanelPartitionSymbolCtx>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SidepanelCtx {
  partitions: Vec<SidepanelPartitionCtx>,
}

impl SidepanelCtx {
  pub const TEMPLATE: &'static str = "sidepanel";

  pub fn new(
    ctx: &GenerateCtx,
    partitions: &Partition,
    file: &ShortPath,
    symbol: &str,
  ) -> Self {
    let partitions = partitions
      .into_iter()
      .map(|(name, nodes)| {
        let mut grouped_nodes = IndexMap::new();

        for node in nodes {
          let name = if !node.ns_qualifiers.is_empty() {
            format!("{}.{}", node.ns_qualifiers.join("."), node.get_name())
          } else {
            node.get_name().to_string()
          };

          let entry = grouped_nodes.entry(name).or_insert(vec![]);
          entry.push(node);
        }

        let symbols = grouped_nodes
          .into_iter()
          .map(|(node_name, nodes)| {
            SidepanelPartitionSymbolCtx::new(
              &nodes,
              symbol == node_name,
              ctx.href_resolver.resolve_path(
                UrlResolveKind::Symbol { file, symbol },
                UrlResolveKind::Symbol {
                  file: &nodes[0].origin,
                  symbol: &node_name,
                },
              ),
              node_name,
            )
          })
          .collect::<Vec<_>>();
        SidepanelPartitionCtx {
          name: name.clone(),
          symbols,
        }
      })
      .collect();

    Self { partitions }
  }
}

#[derive(Debug, Serialize, Clone)]
struct IndexSidepanelFileCtx {
  name: String,
  href: String,
  active: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct IndexSidepanelCtx {
  root_url: String,
  all_symbols_url: Option<String>,
  kind_partitions: Vec<SidepanelPartitionCtx>,
  files: Vec<IndexSidepanelFileCtx>,
}

impl IndexSidepanelCtx {
  pub const TEMPLATE: &'static str = "index_sidepanel";

  pub fn new(
    ctx: &GenerateCtx,
    current_entrypoint: Option<&ModuleSpecifier>,
    doc_nodes_by_url: &super::ContextDocNodesByUrl,
    partitions: Partition,
    current_file: Option<&ShortPath>,
  ) -> Self {
    let files = doc_nodes_by_url
      .keys()
      .filter(|url| {
        ctx
          .main_entrypoint
          .as_ref()
          .map(|main_entrypoint| *url != main_entrypoint)
          .unwrap_or(true)
      })
      .map(|url| {
        let short_path = ctx.url_to_short_path(url);
        IndexSidepanelFileCtx {
          href: ctx.href_resolver.resolve_path(
            current_file.map_or(UrlResolveKind::Root, UrlResolveKind::File),
            if ctx.main_entrypoint.is_some()
              && ctx.main_entrypoint.as_ref() == Some(url)
            {
              UrlResolveKind::Root
            } else {
              UrlResolveKind::File(&short_path)
            },
          ),
          name: short_path.to_name(),
          active: current_entrypoint
            .is_some_and(|current_entrypoint| current_entrypoint == url),
        }
      })
      .collect::<Vec<_>>();

    let kind_partitions = partitions
      .into_iter()
      .map(|(name, nodes)| {
        let mut grouped_nodes = IndexMap::new();

        for node in &nodes {
          let name = if !node.ns_qualifiers.is_empty() {
            format!("{}.{}", node.ns_qualifiers.join("."), node.get_name())
          } else {
            node.get_name().to_string()
          };

          let entry = grouped_nodes.entry(name).or_insert(vec![]);
          entry.push(node);
        }

        let symbols = grouped_nodes
          .into_iter()
          .map(|(node_name, nodes)| {
            SidepanelPartitionSymbolCtx::new(
              &nodes,
              false,
              ctx.href_resolver.resolve_path(
                current_file.map_or(UrlResolveKind::Root, UrlResolveKind::File),
                UrlResolveKind::Symbol {
                  file: &nodes[0].origin,
                  symbol: &node_name,
                },
              ),
              node_name,
            )
          })
          .collect::<Vec<_>>();
        SidepanelPartitionCtx { name, symbols }
      })
      .collect::<Vec<_>>();

    Self {
      root_url: ctx.href_resolver.resolve_path(
        current_file.map_or(UrlResolveKind::Root, UrlResolveKind::File),
        UrlResolveKind::Root,
      ),
      all_symbols_url: (!ctx.sidebar_hide_all_symbols).then(|| {
        ctx.href_resolver.resolve_path(
          current_file.map_or(UrlResolveKind::Root, UrlResolveKind::File),
          UrlResolveKind::AllSymbols,
        )
      }),
      kind_partitions,
      files,
    }
  }
}
