---
title: Quick Start
description: Get up and running with Assura in minutes
---

import { Steps } from '@astrojs/starlight/components';

Get started with Assura in just a few simple steps.

<Steps>

1. **Install Assura**

   ```bash
   cargo install assura
   ```

2. **Create a Configuration File**

   Create an `assura.yaml` file in your project root:

   ```yaml
   name: My Project
   version: "1.0"
   
   rules:
     - name: file-naming
       severity: high
       pattern: "^[a-z][a-z0-9_]*\\.(rs|toml)$"
   ```

3. **Run Validation**

   ```bash
   assura validate
   ```

4. **Enable Watch Mode (Optional)**

   For continuous validation during development:

   ```bash
   assura watch
   ```

</Steps>

## Next Steps

- Learn about [configuration options](/docs/configuration/)
- Explore [available rules](/docs/rules/)
- Check out [usage examples](/examples/basic/)
