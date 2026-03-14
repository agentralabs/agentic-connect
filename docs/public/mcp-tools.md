# AgenticConnect MCP Tools Reference

## Protocol Tools (Capability 1)

| Tool | Description |
|------|-------------|
| `connect_protocol_detect` | Detect protocol from URL or host:port |
| `connect_protocol_list` | List all supported protocols with capabilities |
| `connect_protocol_test` | Test if a protocol endpoint is reachable |
| `connect_protocol_caps` | Get capabilities of a specific protocol |

## Auth Tools (Capability 2)

| Tool | Description |
|------|-------------|
| `connect_auth_configure` | Configure authentication for a connection |
| `connect_auth_test` | Test if authentication credentials are valid |
| `connect_auth_refresh` | Force token refresh for OAuth2 connections |
| `connect_auth_rotate` | Rotate credentials for a connection |
| `connect_auth_vault` | Manage the encrypted credential vault |

## Connection Soul Tools (Capability 3)

| Tool | Description |
|------|-------------|
| `connect_soul_inspect` | View accumulated knowledge about a connection |
| `connect_soul_refresh` | Force re-scan of remote system capabilities |
| `connect_soul_history` | View connection history and patterns |
| `connect_soul_predict` | Predict likely issues based on past patterns |
| `connect_soul_compare` | Compare two systems for migration planning |

## Retry Tools (Capability 4)

| Tool | Description |
|------|-------------|
| `connect_retry_configure` | Configure retry policies per connection |
| `connect_retry_status` | View retry and circuit breaker state |
| `connect_retry_patterns` | View learned failure patterns |
| `connect_retry_circuit` | View or reset circuit breaker state |
| `connect_retry_simulate` | Simulate failure to test classification |

## Browser Tools (Capabilities 5-7)

| Tool | Description |
|------|-------------|
| `connect_browse_navigate` | Navigate to URL with full page load |
| `connect_browse_understand` | Get semantic understanding of current page |
| `connect_browse_interact` | Interact with page by intent |
| `connect_browse_state` | Get current page state |
| `connect_browse_screenshot` | Capture page screenshot |
| `connect_browse_wait` | Wait for condition on page |
| `connect_scrape_extract` | Extract structured data from page |
| `connect_scrape_monitor` | Monitor page for changes |
| `connect_scrape_adapt` | Adapt extractors to layout changes |
| `connect_scrape_batch` | Scrape multiple pages with throttling |
| `connect_scrape_history` | View extraction history |
| `connect_form_analyze` | Analyze form structure and fields |
| `connect_form_fill` | Fill form fields with data |
| `connect_form_submit` | Submit form and capture result |
| `connect_form_wizard` | Navigate multi-step form flow |
| `connect_form_validate` | Check form for validation errors |

## API Tools (Capabilities 8-9)

| Tool | Description |
|------|-------------|
| `connect_api_discover` | Discover API endpoints from base URL |
| `connect_api_spec` | Parse API specification (OpenAPI/Swagger) |
| `connect_api_call` | Make HTTP API call with auth and retry |
| `connect_api_profile` | View behavioral profile of an API |
| `connect_api_mock` | Generate mock server from API profile |
| `connect_api_test` | Test API endpoints against expectations |
| `connect_graphql_introspect` | Discover GraphQL schema |
| `connect_graphql_query` | Execute GraphQL query |
| `connect_graphql_build` | Build query from natural language |
| `connect_graphql_subscribe` | Manage real-time subscriptions |
| `connect_graphql_normalize` | Flatten nested response data |

## Infrastructure Tools (Capabilities 10-12)

| Tool | Description |
|------|-------------|
| `connect_remote_exec` | Execute command with pre/post checks |
| `connect_remote_plan` | Create multi-step execution plan |
| `connect_remote_verify` | Verify system state matches expectations |
| `connect_remote_rollback` | Execute rollback plan |
| `connect_remote_transfer` | Transfer files with verification |
| `connect_remote_tunnel` | Create SSH tunnel |
| `connect_mesh_discover` | Discover services and dependencies |
| `connect_mesh_health` | Health check all services |
| `connect_mesh_trace` | Trace request through service chain |
| `connect_mesh_cascade` | Detect cascading failures |
| `connect_mesh_topology` | View service topology |
| `connect_container_deploy` | Deploy containers to any platform |
| `connect_container_status` | View container status |
| `connect_container_logs` | Stream container logs |
| `connect_container_scale` | Scale containers up/down |
| `connect_container_inspect` | Inspect running container |

## Communication Tools (Capabilities 13-15)

| Tool | Description |
|------|-------------|
| `connect_email_send` | Send email with template support |
| `connect_email_read` | Read inbox with filters |
| `connect_email_thread` | Reconstruct email thread |
| `connect_email_actions` | Extract action items |
| `connect_email_draft` | Draft contextual reply |
| `connect_email_search` | Search emails |
| `connect_phone_call` | Place or manage phone call |
| `connect_phone_voicemail` | Retrieve and transcribe voicemail |
| `connect_phone_ivr` | Navigate IVR menu by intent |
| `connect_phone_conference` | Create/manage conference call |
| `connect_phone_sms` | Send/receive SMS |
| `connect_webhook_register` | Register webhook endpoint |
| `connect_webhook_send` | Send webhook with HMAC signature |
| `connect_webhook_receive` | View received webhooks |
| `connect_webhook_route` | Configure routing rules |
| `connect_webhook_verify` | Verify HMAC-SHA256 signature |

## Data Channel Tools (Capabilities 16-18)

| Tool | Description |
|------|-------------|
| `connect_db_connect` | Connect to database (auto-detect type) |
| `connect_db_query` | Execute SQL query with formatting |
| `connect_db_schema` | Discover database schema |
| `connect_db_health` | Monitor database health |
| `connect_db_optimize` | Suggest query optimizations (EXPLAIN) |
| `connect_db_migrate` | Detect schema changes |
| `connect_queue_publish` | Publish to any queue |
| `connect_queue_consume` | Consume from any queue |
| `connect_queue_status` | View queue depth/throughput |
| `connect_queue_dead_letter` | View/replay dead letters |
| `connect_queue_configure` | Configure queue connection |
| `connect_cloud_storage` | Object storage (any cloud) |
| `connect_cloud_compute` | VM operations (any cloud) |
| `connect_cloud_detect` | Detect cloud provider |
| `connect_cloud_cost` | Estimate operation cost |
| `connect_cloud_resource` | List cloud resources |

## Security Tools (Capabilities 19-20)

| Tool | Description |
|------|-------------|
| `connect_tls_inspect` | Inspect TLS configuration |
| `connect_tls_certificates` | Monitor certificate expiry |
| `connect_tls_audit` | Audit cipher suites |
| `connect_tls_expiry` | Check cert expiry for multiple hosts |
| `connect_tls_grade` | Grade TLS quality (A-F) |
| `connect_sentinel_probe` | Health check any endpoint |
| `connect_sentinel_monitor` | Continuous monitoring |
| `connect_sentinel_status` | Aggregate health status |
| `connect_sentinel_trends` | Performance trends |
| `connect_sentinel_alert` | Configure alert thresholds |
| `connect_sentinel_report` | Availability report |

## Intelligence Tools (Capabilities 21-24)

| Tool | Description |
|------|-------------|
| `connect_prophecy_predict` | Predict connection failures |
| `connect_prophecy_trends` | View concerning trends |
| `connect_prophecy_recommend` | Proactive recommendations |
| `connect_prophecy_simulate` | Simulate failure scenarios |
| `connect_evolve_track` | Track API schema over time |
| `connect_evolve_drift` | Detect schema drift |
| `connect_evolve_deprecated` | Identify deprecated fields |
| `connect_evolve_breaking` | Detect breaking changes |
| `connect_evolve_adapt` | Auto-adapt to API changes |
| `connect_dream_start` | Start idle maintenance |
| `connect_dream_insights` | Get idle check discoveries |
| `connect_dream_refresh` | Refresh all profiles |
| `connect_dream_health` | Proactive health report |
| `connect_collective_share` | Share learned API behavior |
| `connect_collective_search` | Search known behaviors |
| `connect_collective_apply` | Apply community knowledge |
| `connect_collective_rate` | Rate knowledge accuracy |
| `connect_collective_private` | Verify no private data shared |
