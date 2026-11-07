# SAMSARA character sheet manager

Ficha minimalista em Rust + Leptos (CSR/WebAssembly), HTML e CSS. Sem imagens e sem JavaScript escrito manualmente.

## Rodar localmente

Instale Rust, Trunk e o target WASM:

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve --open
```

## Publicar no GitHub Pages

1. Crie um repositório GitHub (ex.: `etheris-web`).
2. Envie os arquivos desta pasta para a branch `main`.
3. Em **Settings → Pages → Build and deployment → Source**, selecione **GitHub Actions**.
4. Aguarde a execução de **Actions → Publicar no GitHub Pages**.
5. O endereço será `https://SEU-USUARIO.github.io/etheris-web/` (ajuste para o nome real do repositório).

## Limitações importantes

- Esta é a primeira versão **local**, sem autenticação, banco de dados ou sincronização.
- Os dados são guardados no `localStorage` do navegador. Outro dispositivo ou jogador não vê a mesma ficha.
- GitHub Pages hospeda o frontend, não um backend Rust persistente.
- Não coloque segredos nem credenciais de banco no código publicado.
- Para cada jogador editar apenas sua ficha e o mestre ver todas, a próxima etapa exige autenticação, banco de dados e regras de acesso (ex.: Supabase Auth + RLS).
- Campos e valores são manuais; regras ainda incompletas do ETHERIS não são calculadas automaticamente.
