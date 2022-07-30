#![allow(unused)]
use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::Result;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

pub struct DrawIo;

impl Preprocessor for DrawIo {
    fn name(&self) -> &str {
        "drawio"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        
        let mut res = None;
        book.for_each_mut(|item: &mut BookItem| {
            if let Some(Err(_)) = res {
                return;
            }

            if let BookItem::Chapter(ref mut chapter) = *item {
                res = Some(DrawIo::add_diagram(chapter).map(|md| {
                    chapter.content = md;
                }));
            }
        });
        res.unwrap_or(Ok(())).map(|_| book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}


impl DrawIo {
    fn add_diagram(chapter: &mut Chapter) -> Result<String> {
        Ok("hello world".to_string())
    }
}
