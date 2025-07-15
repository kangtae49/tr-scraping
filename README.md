# Tauri + React + Typescript

This template should help get you started developing with Tauri, React and Typescript in Vite.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

```sh
❯ pnpm create tauri-app
✔ Project name · tr-crawler
✔ Identifier · com.tr-crawler.app
✔ Choose which language to use for your frontend · TypeScript / JavaScript - (pnpm, yarn, npm, deno, bun)
✔ Choose your package manager · pnpm
✔ Choose your UI template · React - (https://react.dev/)
✔ Choose your UI flavor · TypeScript

Template created! To get started run:
  cd tr-crawler
  pnpm install
  pnpm tauri android init

For Desktop development, run:
  pnpm tauri dev

For Android development, run:
  pnpm tauri android dev
```

```sh
pnpm tauri dev -- -- "C:\sources\crawler_data\crawler.json"
```

```sh
pnpm add monaco-editor
```

## publish

```sh
pnpm tauri build
```

## crawler.json

```json
{
  "$schema": "./setting.schema.json",
  "env": {
    "CAFE_ID": "26989041",
    "COOKIE": "XXX"
  },
  "header": {
    "priority": "u=1, i",
    "sec-ch-ua": "\"Google Chrome\";v=\"137\", \"Chromium\";v=\"137\", \"Not/A)Brand\";v=\"24\"",
    "sec-ch-ua-mobile": "?0",
    "sec-ch-ua-platform": "\"Windows\"",
    "sec-fetch-dest": "empty",
    "sec-fetch-mode": "cors",
    "sec-fetch-site": "same-site",
    "x-cafe-product": "pc",
    "Accept": "application/json",
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36",
    "Origin": "https://cafe.naver.com",
    "Cookie": "{{{COOKIE}}}"
  },
  "steps": {
    "menu": {
      "name": "menu",
      "task_iters": [],
      "job": {"HttpJob": {
        "method": "GET",
        "url": "https://apis.naver.com/cafe-web/cafe-cafemain-api/v1.0/cafes/{{CAFE_ID}}/menus",
        "header": {
          "Referer": "https://cafe.naver.com/f-e/cafes/{{CAFE_ID}}/menus/1?viewType=L"
        },
        "filename": "menu_{{CAFE_ID}}.json",
        "output": "C:/sources/crawler_data/menu"
      }},
      "concurrency_limit": 10
    },
    "step2": {
      "name": "step2",
      "task_iters": [
        {
          "GlobJsonPattern": {
            "glob_pattern": "C:/sources/crawler_data/menu/*.json",
            "item_pattern": "$.result.menus[*]",
            "env_pattern": {
              "MENU_ID": "$.menuId"
            }
          }
        }
      ],
      "job": { "HttpJob": {
        "method": "GET",
        "url": "https://apis.naver.com/cafe-web/cafe-boardlist-api/v1/cafes/{{CAFE_ID}}/menus/{{MENU_ID}}/articles?page=1&pageSize=15&sortBy=TIME&viewType=L",
        "header": {
          "Referer": "https://cafe.naver.com/f-e/cafes/{{CAFE_ID}}/menus/1?viewType=L"
        },
        "filename": "page_size_{{CAFE_ID}}_{{MENU_ID}}.json",
        "output": "C:/sources/crawler_data/step2"
      }},
      "concurrency_limit": 10
    },
    "step3": {
      "name": "step3",
      "task_iters": [
        {
          "GlobJsonPattern": {
            "glob_pattern": "C:/sources/crawler_data/menu/*.json",
            "item_pattern": "$.result.menus[*]",
            "env_pattern": {
              "MENU_ID": "$.menuId",
              "MENU_NAME": "$.name"
            }
          }
        },
        {
          "GlobJsonRangePattern": {
            "name": "PAGE_NO",
            "file_pattern": "C:/sources/crawler_data/step2/page_size_{{CAFE_ID}}_{{MENU_ID}}.json",
            "offset_pattern": "1",
            "take_pattern": "$.result.pageInfo.lastNavigationPageNumber"
          }
        }
      ],
      "job": {"HttpJob": {
        "method": "GET",
        "url": "https://apis.naver.com/cafe-web/cafe-boardlist-api/v1/cafes/{{CAFE_ID}}/menus/{{MENU_ID}}/articles?page={{PAGE_NO}}&pageSize=15&sortBy=TIME&viewType=L",
        "header": {
          "Referer": "https://cafe.naver.com/f-e/cafes/{{CAFE_ID}}/menus/1?viewType=L"
        },
        "filename": "page_{{CAFE_ID}}_{{MENU_ID}}_{{PAGE_NO}}.json",
        "output": "C:/sources/crawler_data/step3"
      }},
      "concurrency_limit": 10
    },
    "article": {
      "name": "article",
      "task_iters": [
        {
          "GlobJsonPattern": {
            "glob_pattern": "C:/sources/crawler_data/menu/*.json",
            "item_pattern": "$.result.menus[*]",
            "env_pattern": {
              "MENU_ID": "$.menuId",
              "MENU_NAME": "$.name"
            }
          }
        },
        {
          "GlobJsonPattern": {
            "glob_pattern": "C:/sources/crawler_data/step3/page_{{CAFE_ID}}_{{MENU_ID}}_*.json",
            "item_pattern": "$.result.articleList[*]",
            "env_pattern": {
              "ARTICLE_ID": "$.item.articleId",
              "SUBJECT": "$.item.subject"
            }
          }
        }
      ],
      "job": {"HttpJob": {
        "method": "GET",
        "url": "https://apis.naver.com/cafe-web/cafe-articleapi/v3/cafes/{{CAFE_ID}}/articles/{{ARTICLE_ID}}?query=&menuId={{MENU_ID}}&useCafeId=true&requestFrom=A",
        "header": {
          "Referer": "https://cafe.naver.com/ca-fe/cafes/{{CAFE_ID}}/articles/{{ARTICLE_ID}}?menuid={{MENU_ID}}&referrerAllArticles=false&fromNext=true"
        },
        "filename": "{{MENU_ID}}_{{ARTICLE_ID}}_{{SUBJECT}}.json",
        "output": "C:/sources/crawler_data/article/{{MENU_ID}}_{{MENU_NAME}}"
      }},
      "concurrency_limit": 10
    },
    "article_tsv": {
      "name": "article_tsv",
      "task_iters": [
        {
          "GlobJsonPattern": {
            "glob_pattern": "C:/sources/crawler_data/menu/*.json",
            "item_pattern": "$.result.menus[*]",
            "env_pattern": {
              "MENU_ID": "$.menuId",
              "MENU_NAME": "$.name"
            }
          }
        },
        {
          "GlobJsonPattern": {
            "glob_pattern": "C:/sources/crawler_data/step3/page_{{CAFE_ID}}_{{MENU_ID}}_*.json",
            "item_pattern": "$.result.articleList[*]",
            "env_pattern": {
              "ARTICLE_ID": "$.item.articleId",
              "SUBJECT": "$.item.subject"
            }
          }
        }
      ],
      "job": {"CsvJob": {
        "keys": ["MENU_ID", "ARTICLE_ID", "SUBJECT"],
        "sep": "\t",
        "filename": "{{MENU_ID}}.tsv",
        "output": "C:/sources/crawler_data/articles"
      }},
      "concurrency_limit": 10
    },
    "attachment": {
      "name": "attachment",
      "task_iters": [
        {
          "GlobJsonPattern": {
            "glob_pattern": "C:/sources/crawler_data/menu/*.json",
            "item_pattern": "$.result.menus[*]",
            "env_pattern": {
              "MENU_ID": "$.menuId",
              "MENU_NAME": "$.name"
            }
          }
        },
        {
          "GlobJsonPattern": {
            "glob_pattern": "C:/sources/crawler_data/article/{{MENU_ID}}_{{MENU_NAME}}/{{MENU_ID}}_*.json",
            "item_pattern": "$.result",
            "env_pattern": {
              "ARTICLE_ID": "$.articleId"
            }
          }
        },
        {
          "GlobJsonPattern": {
            "glob_pattern": "C:/sources/crawler_data/article/{{MENU_ID}}_{{MENU_NAME}}/{{MENU_ID}}_{{ARTICLE_ID}}_*.json",
            "item_pattern": "$.result.attaches[*]",
            "env_pattern": {
              "URL": "$.url",
              "FILE_NAME": "$.name"
            }
          }
        }
      ],
      "job": {"HttpJob": {
        "method": "GET",
        "url": "{{{URL}}}",
        "header": {
          "Referer": "https://cafe.naver.com/ca-fe/cafes/{{CAFE_ID}}/articles/{{ARTICLE_ID}}?menuid={{MENU_ID}}&referrerAllArticles=false&fromNext=true",
          "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
          "priority": "u=0, i",
          "sec-ch-ua": "\"Not)A;Brand\";v=\"8\", \"Chromium\";v=\"138\", \"Google Chrome\";v=\"138\"",
          "sec-ch-ua-mobile": "?0",
          "sec-ch-ua-platform": "\"Windows\"",
          "sec-fetch-dest": "iframe",
          "sec-fetch-mode": "navigate",
          "sec-fetch-site": "cross-site",
          "sec-fetch-storage-access": "active",
          "sec-fetch-user": "?1",
          "upgrade-insecure-requests": "1",
          "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36"
        },
        "filename": "{{MENU_ID}}_{{ARTICLE_ID}}_{{FILE_NAME}}",
        "output": "C:/sources/crawler_data/attachment/{{MENU_ID}}_{{MENU_NAME}}"
      }},
      "concurrency_limit": 10
    },
    "output_html": {
      "name": "output_html",
      "task_iters": [
        {
          "GlobJsonPattern": {
            "glob_pattern": "C:/sources/crawler_data/menu/*.json",
            "item_pattern": "$.result.menus[*]",
            "env_pattern": {
              "MENU_ID": "$.menuId",
              "MENU_NAME": "$.name"
            }
          }
        },
        {
          "GlobJsonPattern": {
            "glob_pattern": "C:/sources/crawler_data/article/{{MENU_ID}}_*/*.json",
            "item_pattern": "$.result",
            "env_pattern": {
              "ARTICLE_ID": "$.articleId",
              "WRITE_DATE": "$.article.writeDate",
              "SUBJECT": "$.article.subject",
              "CONTENT_HTML": "$.article.contentHtml",
              "COMMENTS": "$.comments.items",
              "ATTACHES": "$.attaches"
            }
          }
        }
      ],
      "job": {"HtmlJob": {
        "json_map": {
          "COMMENTS": [
            ["ID", "$.writer.id"],
            ["NICK", "$.writer.nick"],
            ["UPDATE_DATE", "$.updateDate"],
            ["CONTENT", "$.content"]
          ],
          "ATTACHES": [
            ["_NAME", "$.name"],
            ["ATTACH", "{{MENU_ID}}_{{ARTICLE_ID}}_{{_NAME}}"]
          ]
        },
        "filename": "{{MENU_ID}}_{{ARTICLE_ID}}_{{SUBJECT}}.html",
        "output_template_file": "C:/sources/tr-crawler/sample/output_template.html",
        "output": "C:/sources/crawler_data/html/{{MENU_ID}}_{{MENU_NAME}}"
      }},
      "concurrency_limit": 10
    },
    "cmd": {
      "name": "cmd",
      "job": {"ShellJob": {
        "shell": "powershell",
        "args": ["-Command", "ls"],
        "encoding": "windows-949",
        "working_dir": "."
      }},
      "task_iters": [],
      "concurrency_limit": 1
    }
  }
}






```