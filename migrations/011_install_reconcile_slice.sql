-- Migration 011 - install.reconcile slice.
--
-- Minimal installation substrate for the first Reconcile-shaped
-- constitutional slice. Desired and observed state are intentionally JSONB:
-- the slice proves convergence anatomy without pretending to have the final
-- payload/service model.
--
-- Evidence remains in the single evidence_ledger:
--   install.reconcile.planned
--   install.reconcile.step.applied
--   install.reconcile.reconciled
--   install.reconcile.failed

create table if not exists installation (
    id uuid primary key,
    host_id uuid references host (id) on delete set null,
    desired_manifest jsonb not null default '{}'::jsonb,
    observed_manifest jsonb not null default '{}'::jsonb,
    status text not null default 'pending',
    canon_version text not null default 'unknown',
    elastic_version text not null default 'unknown',
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create index if not exists installation_host_idx on installation (host_id);
create index if not exists installation_status_idx on installation (status);

comment on table installation is
    'Minimal desired/observed substrate for the install.reconcile constitutional slice.';
comment on column installation.desired_manifest is
    'Desired install state. Current slice expects a top-level services object.';
comment on column installation.observed_manifest is
    'Observed install state. Current slice expects a top-level services object.';
