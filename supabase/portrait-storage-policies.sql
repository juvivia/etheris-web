-- Policies adicionais para retratos de fichas individuais.
-- O bucket "portraits" permanece privado. O caminho usado pelo app é:
-- {character_id}/{user_id}/portrait.{jpg|png|webp}

create policy "Character owners can view their portraits"
on storage.objects
for select
to authenticated
using (
    bucket_id = 'portraits'
    and (storage.foldername(name))[2] = auth.uid()::text
    and exists (
        select 1
        from public.characters c
        where c.id::text = (storage.foldername(name))[1]
          and c.user_id = auth.uid()
    )
);

create policy "Character owners can upload their portraits"
on storage.objects
for insert
to authenticated
with check (
    bucket_id = 'portraits'
    and (storage.foldername(name))[2] = auth.uid()::text
    and exists (
        select 1
        from public.characters c
        where c.id::text = (storage.foldername(name))[1]
          and c.user_id = auth.uid()
    )
);

create policy "Character owners can update their portraits"
on storage.objects
for update
to authenticated
using (
    bucket_id = 'portraits'
    and (storage.foldername(name))[2] = auth.uid()::text
    and exists (
        select 1
        from public.characters c
        where c.id::text = (storage.foldername(name))[1]
          and c.user_id = auth.uid()
    )
)
with check (
    bucket_id = 'portraits'
    and (storage.foldername(name))[2] = auth.uid()::text
    and exists (
        select 1
        from public.characters c
        where c.id::text = (storage.foldername(name))[1]
          and c.user_id = auth.uid()
    )
);
