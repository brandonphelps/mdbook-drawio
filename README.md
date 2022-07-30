# Motiviation

This intends to be a preprocessor for the mdbook project that extends the ability
to render drawio diagrams into the mdbook output. 

This project builds on top of [https://github.com/rlespinasse/docker-drawio-desktop-headless](https://github.com/rlespinasse/docker-drawio-desktop-headless) to generate the final images. 

As such docker is required for transforming the drawio diagram from .drawio to svg. 

# Requirements

- Docker
- mdbook
- linux

# Usage

Inside the your mdbook name links such as 

```
![link-name](<diagram_path>-<page>.drawio)
```

This will result in the diagram found at `diagram_path` being generated into multiple svgs, 1 per page. 

# Errors

If there is an error in converting the document then the .svg will not be found and as such 
the link in the rendered document will just look like some text or defaults back what a broken link will look like. 

