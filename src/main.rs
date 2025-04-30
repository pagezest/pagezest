// Blog Posts
// Create Post
// Read Post
// Read All Posts
mod api;
mod db;
mod errors;
mod memory;
mod mime;
mod plugin;
mod post;
mod server;
mod plugin_manager;
mod render;

use rusqlite::Connection;
use serde_json::json;

use crate::errors::AppError;
use crate::memory::get_process_memory;
use crate::post::BlogPost;

fn main() -> Result<(), AppError> {
    let m1 = get_process_memory();
    // Initializing DB for blog posts.
    let conn = Connection::open("pagezest.db")?;
    db::init_db(&conn)?;
    let m2 = get_process_memory();
    // If no blogs are there then create one sample blog.
    if db::get_all_post(&conn)?.is_empty() {
        let blog_post = BlogPost::new(
            "",
            "PageZest Example Blog",
            json!(
                {"json":[{"raw":"[[toc]] [[/toc]]","text":"[[toc]] [[/toc]]","tokens":[{"escaped":false,"raw":"[[toc]] [[/toc]]","text":"[[toc]] [[/toc]]","type":"text"}],"type":"paragraph"},{"raw":"\n\n","type":"space"},{"depth":1,"raw":"# Heading1\n","text":"Heading1","tokens":[{"escaped":false,"raw":"Heading1","text":"Heading1","type":"text"}],"type":"heading"},{"raw":"`code`\n*strong*  **italic** and ~strikethrough~","text":"`code`\n*strong*  **italic** and ~strikethrough~","tokens":[{"raw":"`code`","text":"code","type":"codespan"},{"escaped":false,"raw":"\n","text":"\n","type":"text"},{"raw":"*strong*","text":"strong","tokens":[{"escaped":false,"raw":"strong","text":"strong","type":"text"}],"type":"em"},{"escaped":false,"raw":"  ","text":"  ","type":"text"},{"raw":"**italic**","text":"italic","tokens":[{"escaped":false,"raw":"italic","text":"italic","type":"text"}],"type":"strong"},{"escaped":false,"raw":" and ","text":" and ","type":"text"},{"raw":"~strikethrough~","text":"strikethrough","tokens":[{"escaped":false,"raw":"strikethrough","text":"strikethrough","type":"text"}],"type":"del"}],"type":"paragraph"},{"raw":"\n\n","type":"space"},{"lang":"","raw":"```\n{\n  \"firstName\": \"John\",\n  \"lastName\": \"Smith\",\n  \"age\": 25\n}\n``` ","text":"{\n  \"firstName\": \"John\",\n  \"lastName\": \"Smith\",\n  \"age\": 25\n}","type":"code"},{"raw":"\n\n","type":"space"},{"raw":"[pagezest](https://www.pagezest.com)","text":"[pagezest](https://www.pagezest.com)","tokens":[{"href":"https://www.pagezest.com","raw":"[pagezest](https://www.pagezest.com)","text":"pagezest","title":null,"tokens":[{"escaped":false,"raw":"pagezest","text":"pagezest","type":"text"}],"type":"link"}],"type":"paragraph"},{"raw":"\n\n","type":"space"},{"depth":2,"raw":"## List\n","text":"List","tokens":[{"escaped":false,"raw":"List","text":"List","type":"text"}],"type":"heading"},{"items":[{"loose":false,"raw":"* Item 1\n","task":false,"text":"Item 1","tokens":[{"raw":"Item 1","text":"Item 1","tokens":[{"escaped":false,"raw":"Item 1","text":"Item 1","type":"text"}],"type":"text"}],"type":"list_item"},{"loose":false,"raw":"* Item 2\n","task":false,"text":"Item 2","tokens":[{"raw":"Item 2","text":"Item 2","tokens":[{"escaped":false,"raw":"Item 2","text":"Item 2","type":"text"}],"type":"text"}],"type":"list_item"},{"loose":false,"raw":"* Item 3","task":false,"text":"Item 3","tokens":[{"raw":"Item 3","text":"Item 3","tokens":[{"escaped":false,"raw":"Item 3","text":"Item 3","type":"text"}],"type":"text"}],"type":"list_item"}],"loose":false,"ordered":false,"raw":"* Item 1\n* Item 2\n* Item 3","start":"","type":"list"},{"raw":"\n\n","type":"space"},{"depth":1,"raw":"# table\n","text":"table","tokens":[{"escaped":false,"raw":"table","text":"table","type":"text"}],"type":"heading"},{"align":[null,null],"header":[{"align":null,"header":true,"text":"col","tokens":[{"escaped":false,"raw":"col","text":"col","type":"text"}]},{"align":null,"header":true,"text":"col","tokens":[{"escaped":false,"raw":"col","text":"col","type":"text"}]}],"raw":"|col|col|\n|-|-|\n|cell|cell|\n|cell|cell|\n|cell|cell|\n\n","rows":[[{"align":null,"header":false,"text":"cell","tokens":[{"escaped":false,"raw":"cell","text":"cell","type":"text"}]},{"align":null,"header":false,"text":"cell","tokens":[{"escaped":false,"raw":"cell","text":"cell","type":"text"}]}],[{"align":null,"header":false,"text":"cell","tokens":[{"escaped":false,"raw":"cell","text":"cell","type":"text"}]},{"align":null,"header":false,"text":"cell","tokens":[{"escaped":false,"raw":"cell","text":"cell","type":"text"}]}],[{"align":null,"header":false,"text":"cell","tokens":[{"escaped":false,"raw":"cell","text":"cell","type":"text"}]},{"align":null,"header":false,"text":"cell","tokens":[{"escaped":false,"raw":"cell","text":"cell","type":"text"}]}]],"type":"table"}],"md":"[[toc]] [[/toc]]\n\n# Heading1\n`code`\n*strong*  **italic** and ~strikethrough~\n\n```\n{\n  \"firstName\": \"John\",\n  \"lastName\": \"Smith\",\n  \"age\": 25\n}\n``` \n\n[pagezest](https://www.pagezest.com)\n\n## List\n* Item 1\n* Item 2\n* Item 3\n\n# table\n|col|col|\n|-|-|\n|cell|cell|\n|cell|cell|\n|cell|cell|\n\n"}
            ),
        );
        db::create_post(&conn, blog_post).unwrap();
    }
    let m3 = get_process_memory();
    println!("Starting Pagezest Instance");
    println!("Initial Memory at : {} KB", m1);
    println!("DB Initialized Memory : {} KB", m2);
    println!("Sample Post Generated : {} KB", m3);
    // Run a server.use post::BlogPost;
    server::run_server(conn)
}
