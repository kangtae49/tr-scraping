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
        "CAFE_ID": "18432106",
        "COOKIE": "xxx"
    },
    "steps": {
        "step1": {
            "name": "step1",
            "input": {},
            "req": {
                "method": "GET",
                "url": "https://apis.naver.com/cafe-web/cafe-cafemain-api/v1.0/cafes/{{CAFE_ID}}/menus",
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
                    "Referer": "https://cafe.naver.com/f-e/cafes/18432106/menus/1?viewType=L",
                    "Origin": "https://cafe.naver.com",
                    "Cookie": "{{{COOKIE}}}"
                },
                "filename": "menu_{{CAFE_ID}}.json"
            },
            "output": "C:/sources/crawler_data/step1",
            "concurrency_limit": 10
        }

    },
    "edges": []
}

```