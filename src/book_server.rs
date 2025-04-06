use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use serde_json::json;

use tracing_subscriber::{self, EnvFilter};
use std::sync::OnceLock;

use rmcp::{
    Error as McpError, RoleServer, ServerHandler, model::*,
    service::RequestContext, tool,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Book {
    #[schemars(description = "本のタイトル")]
    pub title: String,
    #[schemars(description = "著者名")]
    pub author: String,
    #[schemars(description = "出版年（架空）")]
    pub year: i32,
    #[schemars(description = "本の説明")]
    pub description: String,
    #[schemars(description = "架空のISBN")]
    pub isbn: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchQuery {
    #[schemars(description = "検索キーワード")]
    pub keyword: String,
    #[schemars(description = "最大結果数")]
    pub limit: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct BookSearch;

static FAKE_BOOKS: OnceLock<Vec<Book>> = OnceLock::new();

fn get_fake_books() -> &'static [Book] {
    FAKE_BOOKS.get_or_init(|| {
        vec![
            Book {
                title: "量子コンピュータで料理する方法".to_string(),
                author: "Dr. スーパーサイエンティスト".to_string(),
                year: 2157,
                description: "量子コンピュータを使用して、分子レベルで料理を再構築する革新的な方法を解説".to_string(),
                isbn: "978-0-123456-47-11".to_string(),
            },
            Book {
                title: "タイムトラベルと税金対策".to_string(),
                author: "未来の会計士".to_string(),
                year: 3000,
                description: "タイムトラベルを活用した効率的な税金対策を解説".to_string(),
                isbn: "978-0-123456-47-12".to_string(),
            },
            Book {
                title: "火星での園芸入門".to_string(),
                author: "火星の園芸家".to_string(),
                year: 2250,
                description: "火星の特殊な環境で植物を育てる方法を解説。".to_string(),
                isbn: "978-0-123456-47-13".to_string(),
            },
            Book {
                title: "AIと恋愛の心理学".to_string(),
                author: "ロボット心理学者".to_string(),
                year: 2200,
                description: "AIとの恋愛関係における心理学的な考察と実践的なアドバイス。".to_string(),
                isbn: "978-0-123456-47-14".to_string(),
            },
            Book {
                title: "テレパシーでプログラミング".to_string(),
                author: "サイキックエンジニア".to_string(),
                year: 2300,
                description: "テレパシー能力を使用してコードを書く方法を解説。".to_string(),
                isbn: "978-0-123456-47-15".to_string(),
            },
        ]
    })
}

#[tool(tool_box)]
impl BookSearch {
    pub fn new() -> Self {
        Self
    }

    /// 架空の本を検索するツール
    /// 
    /// # 引数
    /// * SearchQuery - 検索クエリを含むリクエスト構造体
    /// 
    /// # 戻り値
    /// * Result<CallToolResult, McpError> - 検索結果
    #[tool(description = "Search for book in our fictional database")]
    fn search(&self, #[tool(aggr)] SearchQuery { keyword, limit}: SearchQuery) -> Result<CallToolResult, McpError> {
        let limit = limit.unwrap_or(5) as usize;
        let results: Vec<_> = get_fake_books()
            .iter()
            .filter(|book| {
                book.title.to_lowercase().contains(&keyword.to_lowercase()) ||
                book.author.to_lowercase().contains(&keyword.to_lowercase()) ||
                book.description.to_lowercase().contains(&keyword.to_lowercase())
            })
            .take(limit)
            .collect();

        let output = if results.is_empty() {
            format!("キーワード '{}' に一致する本が見つかりませんでした。", keyword)
        } else {
            let mut output = format!("キーワード '{}' の検索結果:\n\n", keyword);
            for book in results {
                output.push_str(&format!(
                    "タイトル: {}\n著者: {}\n出版年: {}\nISBN: {}\n説明: {}\n\n",
                    book.title, book.author, book.year, book.isbn, book.description
                ));
            }
            output
        };

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }
}

#[tool(tool_box)]
impl ServerHandler for BookSearch {
    fn get_info(&self)  -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("架空の本のデータベースを検索するサーバーです。タイトル、著者、説明文で検索できます。".into()),
        }
    }

    async fn list_resources(
        &self,
        _request: PaginatedRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        Err(McpError::resource_not_found(
            "resource_not_found",
            Some(json!({
                "uri": uri
            }))
        ))
    }

    async fn list_prompts(
        &self,
        _request: PaginatedRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            next_cursor: None,
            prompts: vec![],
        })
    }

    async fn get_prompt(
        &self,
        GetPromptRequestParam { name, arguments: _}: GetPromptRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        Err(McpError::invalid_params("prompt not found", None))
    }

    async fn list_resource_templates(
        &self,
        _request: PaginatedRequestParam,
        _: RequestContext<RoleServer>
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: Vec::new(),
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
    .with_writer(std::io::stderr)
    .with_ansi(false)
    .init();

    tracing::info!("Starting MCP book search server");

    let service = BookSearch::new().serve(stdio()).await.inspect_err(|e| {
        tracing::error!("servign error: {:?}", e);
    })?;

    service.waiting().await?;
    Ok(())
}
