# Allfeat Node Helm Chart

This Helm chart simplifies the deployment of an Allfeat node (Substrate blockchain) on Kubernetes.

## Prerequisites

- Kubernetes 1.19+
- Helm 3.2.0+
- Persistent storage configured (recommended for production)

## Installation

### Quick installation

```bash
helm install allfeat-node ./allfeat-node
```

### Installation with custom values

```bash
helm install allfeat-node ./allfeat-node -f values.yaml
```

### Installation from repository

```bash
helm repo add allfeat https://charts.allfeat.com
helm repo update
helm install allfeat-node allfeat/allfeat-node
```

## Configuration

### Deployment modes

#### Development Mode

```yaml
node:
  devMode: true
```

#### Production Mode

```yaml
node:
  devMode: false
  chain: "mainnet" # or your custom chain
  name: "my-allfeat-node"
```

### Storage

#### Persistent storage configuration

```yaml
persistence:
  enabled: true
  storageClass: "fast-ssd"
  size: 500Gi
```

#### Using existing PVC

```yaml
persistence:
  enabled: true
  existingClaim: "my-existing-pvc"
```

### Network and Services

#### P2P Configuration

```yaml
node:
  network:
    port: 30333
    publicAddr: "/ip4/1.2.3.4/tcp/30333"
    bootnodes: "/ip4/bootnode1/tcp/30333/p2p/12D3...,/ip4/bootnode2/tcp/30333/p2p/12D3..."

p2pService:
  enabled: true
  type: LoadBalancer # or NodePort
```

#### RPC/WebSocket Configuration

```yaml
rpc:
  external: true
  port: 9933
  cors: "all"
  methods: "safe" # or "unsafe" for development

ws:
  external: true
  port: 9944
```

#### Ingress

```yaml
ingress:
  enabled: true
  className: "nginx"
  hosts:
    - host: allfeat-rpc.example.com
      paths:
        - path: /
          pathType: Prefix
          port: 9933
```

### Metrics and Monitoring

#### Prometheus

```yaml
metrics:
  enabled: true
  port: 9615

serviceMonitor:
  enabled: true
  interval: 30s
```

### Security

#### Security configuration

```yaml
securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  readOnlyRootFilesystem: true

networkPolicy:
  enabled: true
```

### Resources

#### Resource configuration

```yaml
resources:
  requests:
    cpu: 1000m
    memory: 2Gi
  limits:
    cpu: 2000m
    memory: 4Gi
```

#### Auto-scaling

```yaml
autoscaling:
  enabled: true
  minReplicas: 1
  maxReplicas: 3
  targetCPUUtilizationPercentage: 80
```

## Deployment Examples

### 1. Development node

```yaml
# dev-values.yaml
node:
  devMode: true
  name: "dev-node"

persistence:
  enabled: false

resources:
  requests:
    cpu: 500m
    memory: 1Gi
  limits:
    cpu: 1000m
    memory: 2Gi
```

```bash
helm install dev-node ./allfeat-node -f dev-values.yaml
```

### 2. Production node

```yaml
# prod-values.yaml
replicaCount: 1

node:
  devMode: false
  chain: "mainnet"
  name: "prod-allfeat-node"
  network:
    port: 30333
    publicAddr: "/ip4/YOUR_PUBLIC_IP/tcp/30333"
    bootnodes: "BOOTNODE_ADDRESSES"

persistence:
  enabled: true
  storageClass: "fast-ssd"
  size: 500Gi

resources:
  requests:
    cpu: 2000m
    memory: 4Gi
  limits:
    cpu: 4000m
    memory: 8Gi

p2pService:
  enabled: true
  type: LoadBalancer

metrics:
  enabled: true

serviceMonitor:
  enabled: true

networkPolicy:
  enabled: true
```

```bash
helm install prod-node ./allfeat-node -f prod-values.yaml
```

### 3. Validator cluster

```yaml
# validator-values.yaml
replicaCount: 3

node:
  devMode: false
  chain: "mainnet"
  customArgs:
    - "--validator"
    - "--name"
    - "validator-${POD_NAME}"

persistence:
  enabled: true
  storageClass: "premium-ssd"
  size: 1Ti

affinity:
  podAntiAffinity:
    requiredDuringSchedulingIgnoredDuringExecution:
      - labelSelector:
          matchExpressions:
            - key: app.kubernetes.io/name
              operator: In
              values:
                - allfeat-node
        topologyKey: kubernetes.io/hostname

resources:
  requests:
    cpu: 4000m
    memory: 8Gi
  limits:
    cpu: 8000m
    memory: 16Gi
```

## Useful Commands

### Check status

```bash
kubectl get pods -l app.kubernetes.io/name=allfeat-node
kubectl logs -f deployment/allfeat-node
```

### Access services

```bash
# Port-forward RPC
kubectl port-forward svc/allfeat-node 9933:9933

# Port-forward WebSocket
kubectl port-forward svc/allfeat-node 9944:9944

# Port-forward Metrics
kubectl port-forward svc/allfeat-node 9615:9615
```

### Update

```bash
helm upgrade allfeat-node ./allfeat-node -f values.yaml
```

### Uninstall

```bash
helm uninstall allfeat-node
```

## Parameters

| Parameter                   | Description               | Default                |
| --------------------------- | ------------------------- | ---------------------- |
| `replicaCount`              | Number of replicas        | `1`                    |
| `image.repository`          | Docker image repository   | `allfeat/allfeat-node` |
| `image.tag`                 | Image tag                 | `latest`               |
| `node.devMode`              | Development mode          | `false`                |
| `node.chain`                | Chain specification       | `local`                |
| `node.name`                 | Node name                 | `allfeat-node`         |
| `persistence.enabled`       | Enable persistent storage | `true`                 |
| `persistence.size`          | Storage size              | `100Gi`                |
| `resources.requests.cpu`    | CPU request               | `1000m`                |
| `resources.requests.memory` | Memory request            | `2Gi`                  |
| `p2pService.enabled`        | P2P service               | `true`                 |
| `metrics.enabled`           | Prometheus metrics        | `true`                 |

For the complete list of parameters, see the `values.yaml` file.

## Troubleshooting

### Pod in CrashLoopBackOff state

1. Check logs: `kubectl logs pod-name`
2. Check available resources
3. Check storage configuration

### P2P network issues

1. Verify P2P service is exposed
2. Check firewall rules
3. Verify configured bootnodes

### Synchronization issues

1. Check network connectivity
2. Check available disk space
3. Check connected peers via metrics

## Support

For help:

- GitHub Issues: https://github.com/allfeat/allfeat/issues
- Documentation: https://docs.allfeat.com
- Email: tech@allfeat.com
