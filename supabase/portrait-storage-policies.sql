-- Execute este arquivo no SQL Editor do Supabase.
-- O bucket "portraits" deve existir e permanecer PRIVADO.
-- O aplicativo grava os arquivos neste formato:
-- {character_id}/{user_id}/portrait.{jpg|png|webp}
--
-- As políticas são recriadas de forma idempotente para permitir reaplicar
-- este script sem erros de "policy already exists".

drop policy if exists "Character owners can view their portraits" on storage.objects;
drop policy if exists "Character owners can upload their portraits" on storage.objects;
drop policy if exists "Character owners can update their portraits" on storage.objects;

create policy "Character owners can view their portraits"
on storage.objects
for select
to authenticated
using (
    bucket_id = 'portraits'
    and (storage.foldername(name))[2] = (select auth.uid())::text
    and exists (
        select 1
        from public.characters as c
        where c.id::text = (storage.foldername(name))[1]
          and c.user_id = (select auth.uid())
    )
);

create policy "Character owners can upload their portraits"
on storage.objects
for insert
to authenticated
with check (
    bucket_id = 'portraits'
    and (storage.foldername(name))[2] = (select auth.uid())::text
    and exists (
        select 1
        from public.characters as c
        where c.id::text = (storage.foldername(name))[1]
          and c.user_id = (select auth.uid())
    )
);

create policy "Character owners can update their portraits"
on storage.objects
for update
to authenticated
using (
    bucket_id = 'portraits'
    and (storage.foldername(name))[2] = (select auth.uid())::text
    and exists (
        select 1
        from public.characters as c
        where c.id::text = (storage.foldername(name))[1]
          and c.user_id = (select auth.uid())
    )
)
with check (
    bucket_id = 'portraits'
    and (storage.foldername(name))[2] = (select auth.uid())::text
    and exists (
        select 1
        from public.characters as c
        where c.id::text = (storage.foldername(name))[1]
          and c.user_id = (select auth.uid())
    )
);
