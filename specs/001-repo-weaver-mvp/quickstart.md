# Quickstart: Repo Weaver

## 1. Install
```bash
brew install repo-weaver
```

## 2. Initialize
```bash
mkdir my-infra
cd my-infra
rw init
```

## 3. Configure
Edit `weaver.yaml`:
```yaml
version: "1.0"
modules:
  - name: "base"
    source: "https://github.com/my/modules"
    ref: "main"

apps:
  - name: "vpc"
    module: "base"
    path: "./vpc"
    inputs:
      cidr: "10.0.0.0/16"
```

## 4. Apply
```bash
rw apply
```
This scaffolds the `vpc` directory with files defined in the `base` module.
