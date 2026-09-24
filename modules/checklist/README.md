# checklist

Official Portaki checklists — guest lists ticked in the booklet, host lists (cleaning, inspections) as dated tasks around each stay.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`checklist`

OCI image: `ghcr.io/portakiapp/portaki-modules-checklist:<semver>`

Host workspace tab: `pathSegment = checklist` (surface `main`).

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | `Checklist`, `ChecklistItem`, `ChecklistCompletion`, `TaskItemState` entities |

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Open guest lists — progress + inline toggles |
| guest | `post-stay.card` | Departure lists on the end-of-stay screen |
| host | `main` | « Vos checklists » + selected list → workspace Save → `updateConfig` |
| host | `checklist` | Stats detail « Checklist » (guest lists) |
| host | `cleaning` | Stats detail « Ménage » (host cleaning lists) |

Manifest also declares the `checklist` / `cleaning` stats tiles and `workspace-timeline-task`.

## Queries and commands

- `listItems` — guest items; `listCompletions` — completed item ids for the current stay
- `completeItem` / `uncompleteItem` — guest toggles (`itemId`)
- `updateConfig` — saves the selected list; `createChecklist { template }`, `deleteChecklist { id }`
- `timelineTasks` — host lists as dated tasks for the stays the platform passes (`<checklistId>:<stayId>`)
- `taskToggle` / `taskComplete` — refused with `photo_required` when a photo item has no photo;
  emit `checklist.task-updated`, `workspace-activity.record`, and `consumables.restocked` when
  « Consommables réassortis » gets ticked
- `statsSummary` — tiles `checklist` and `cleaning`

## Development

```bash
cargo test -p checklist
cd modules/checklist
portaki build --release
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
