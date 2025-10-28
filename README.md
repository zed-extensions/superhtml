# SuperHTML support for Zed

This extension provides support for the [SuperHTML](https://github.com/kristoff-it/superhtml) language and LSP.

## Installation

1. Install extension
2. Add the following to your `settings.json`:

```json
{
  "languages": {
    "HTML": {
      "language_servers": ["vscode-html-language-server", "superhtml"],
      "formatter": { "language_server": { "name": "superhtml" } }
    }
  }
}
```
