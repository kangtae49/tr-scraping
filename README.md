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

## crawler.json
```json
{
  "env": {
    "CAFE_ID": "26989041",
    "COOKIE": "xxxxx"
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
    "step1": {
      "name": "step1",
      "task_iters": [],
      "req": {
        "method": "GET",
        "url": "https://apis.naver.com/cafe-web/cafe-cafemain-api/v1.0/cafes/{{CAFE_ID}}/menus",
        "header": {
          "Referer": "https://cafe.naver.com/f-e/cafes/{{CAFE_ID}}/menus/1?viewType=L"
        },
        "filename": "menu_{{CAFE_ID}}.json"
      },
      "output": "C:/sources/crawler_data/step1",
      "concurrency_limit": 10
    },
    "step2": {
      "name": "step2",
      "task_iters": [
        {
          "GlobJsonPattern": {
            "glob_pattern": "C:/sources/crawler_data/step1/*.json",
            "item_pattern": "$.result.menus[*]",
            "env_pattern": {
              "MENU_ID": "$.menuId"
            }
          }
        }
      ],
      "req": {
        "method": "GET",
        "url": "https://apis.naver.com/cafe-web/cafe-boardlist-api/v1/cafes/{{CAFE_ID}}/menus/{{MENU_ID}}/articles?page=1&pageSize=15&sortBy=TIME&viewType=L",
        "header": {
          "Referer": "https://cafe.naver.com/f-e/cafes/{{CAFE_ID}}/menus/1?viewType=L"
        },
        "filename": "page_size_{{CAFE_ID}}_{{MENU_ID}}.json"
      },
      "output": "C:/sources/crawler_data/step2",
      "concurrency_limit": 10
    },
    "step3": {
      "name": "step3",
      "task_iters": [
        {
          "GlobJsonPattern": {
            "glob_pattern": "C:/sources/crawler_data/step1/*.json",
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
      "req": {
        "method": "GET",
        "url": "https://apis.naver.com/cafe-web/cafe-boardlist-api/v1/cafes/{{CAFE_ID}}/menus/{{MENU_ID}}/articles?page={{PAGE_NO}}&pageSize=15&sortBy=TIME&viewType=L",
        "header": {
          "Referer": "https://cafe.naver.com/f-e/cafes/{{CAFE_ID}}/menus/1?viewType=L"
        },
        "filename": "page_{{CAFE_ID}}_{{MENU_ID}}_{{PAGE_NO}}.json"
      },
      "output": "C:/sources/crawler_data/step3",
      "concurrency_limit": 10
    },
    "article": {
      "name": "article",
      "task_iters": [
        {
          "GlobJsonPattern": {
            "glob_pattern": "C:/sources/crawler_data/step1/*.json",
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
              "ARTICLE_ID": "$.item.articleId"
            }
          }
        }
      ],
      "req": {
        "method": "GET",
        "url": "https://apis.naver.com/cafe-web/cafe-articleapi/v3/cafes/{{CAFE_ID}}/articles/{{ARTICLE_ID}}?query=&menuId={{MENU_ID}}&useCafeId=true&requestFrom=A",
        "header": {
          "Referer": "https://cafe.naver.com/ca-fe/cafes/{{CAFE_ID}}/articles/{{ARTICLE_ID}}?menuid={{MENU_ID}}&referrerAllArticles=false&fromNext=true"
        },
        "filename": "article_{{CAFE_ID}}_{{MENU_ID}}_{{ARTICLE_ID}}.json"
      },
      "output": "C:/sources/crawler_data/article/{{MENU_ID}}_{{MENU_NAME}}",
      "concurrency_limit": 10
    },
    "attachment": {
      "name": "attachment",
      "task_iters": [
        {
          "GlobJsonPattern": {
            "glob_pattern": "C:/sources/crawler_data/step1/*.json",
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
            "item_pattern": "$.result.attaches[*]",
            "env_pattern": {
              "URL": "$.url",
              "FILE_NAME": "$.name"
            }
          }
        }
      ],
      "req": {
        "method": "GET",
        "url": "{{{URL}}}",
        "header": {
          "Referer": "https://cafe.naver.com/ca-fe/cafes/{{CAFE_ID}}/articles/{{ARTICLE_ID}}?menuid={{MENU_ID}}&referrerAllArticles=false&fromNext=true"
        },
        "filename": "article_{{CAFE_ID}}_{{MENU_ID}}_{{ARTICLE_ID}}_attachment_{{FILE_NAME}}"
      },
      "output": "C:/sources/crawler_data/article/{{MENU_ID}}_{{MENU_NAME}}",
      "concurrency_limit": 10
    }


  },
  "edges": []
}



```