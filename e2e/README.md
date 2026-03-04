# E2E Tests

End-to-end tests for the xsnap Docker image (`ghcr.io/maxischmaxi/xsnap:latest`).

## Scenarios

### storybook-basic

Minimal Storybook app with 3 components (Button, Badge, Card) and 10 stories. Verifies that xsnap can start Chrome, navigate pages, take screenshots, and compare them.

**Architecture:**

```text
docker-compose up
  ├─ storybook (Node.js) ── Storybook static build, served on port 8080
  └─ xsnap (Docker Image) ── Chrome → navigates to http://storybook:8080 → screenshots
```

**Run:**

```bash
cd e2e/storybook-basic
./run.sh
```

**First run** creates baseline images in `xsnap/__snapshots__/__base_images__/`. Commit these to version control.

**Subsequent runs** compare against baselines. Exit 0 = pass, exit 1 = fail.
