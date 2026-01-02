# Config

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dollar_schema** | Option<**String**> | JSON schema reference for configuration validation | [optional]
**theme** | Option<**String**> | Theme name to use for the interface | [optional]
**keybinds** | Option<[**models::KeybindsConfig**](KeybindsConfig.md)> |  | [optional]
**log_level** | Option<[**models::LogLevel**](LogLevel.md)> |  | [optional]
**tui** | Option<[**models::ConfigTui**](Config_tui.md)> |  | [optional]
**server** | Option<[**models::ServerConfig**](ServerConfig.md)> |  | [optional]
**command** | Option<[**std::collections::HashMap<String, models::ConfigCommandValue>**](Config_command_value.md)> | Command configuration, see https://opencode.ai/docs/commands | [optional]
**watcher** | Option<[**models::ConfigWatcher**](Config_watcher.md)> |  | [optional]
**plugin** | Option<**Vec<String>**> |  | [optional]
**snapshot** | Option<**bool**> |  | [optional]
**share** | Option<**String**> | Control sharing behavior:'manual' allows manual sharing via commands, 'auto' enables automatic sharing, 'disabled' disables all sharing | [optional]
**autoshare** | Option<**bool**> | @deprecated Use 'share' field instead. Share newly created sessions automatically | [optional]
**autoupdate** | Option<[**models::ConfigAutoupdate**](Config_autoupdate.md)> |  | [optional]
**disabled_providers** | Option<**Vec<String>**> | Disable providers that are loaded automatically | [optional]
**enabled_providers** | Option<**Vec<String>**> | When set, ONLY these providers will be enabled. All other providers will be ignored | [optional]
**model** | Option<**String**> | Model to use in the format of provider/model, eg anthropic/claude-2 | [optional]
**small_model** | Option<**String**> | Small model to use for tasks like title generation in the format of provider/model | [optional]
**default_agent** | Option<**String**> | Default agent to use when none is specified. Must be a primary agent. Falls back to 'build' if not set or if the specified agent is invalid. | [optional]
**username** | Option<**String**> | Custom username to display in conversations instead of system username | [optional]
**mode** | Option<[**models::ConfigMode**](Config_mode.md)> |  | [optional]
**agent** | Option<[**models::ConfigAgent**](Config_agent.md)> |  | [optional]
**provider** | Option<[**std::collections::HashMap<String, models::ProviderConfig>**](ProviderConfig.md)> | Custom provider configurations and model overrides | [optional]
**mcp** | Option<[**std::collections::HashMap<String, models::McpAddRequestConfig>**](mcp_add_request_config.md)> | MCP (Model Context Protocol) server configurations | [optional]
**formatter** | Option<[**models::ConfigFormatter**](Config_formatter.md)> |  | [optional]
**lsp** | Option<[**models::ConfigLsp**](Config_lsp.md)> |  | [optional]
**instructions** | Option<**Vec<String>**> | Additional instruction files or patterns to include | [optional]
**layout** | Option<[**models::LayoutConfig**](LayoutConfig.md)> |  | [optional]
**permission** | Option<[**models::AgentConfigPermission**](AgentConfig_permission.md)> |  | [optional]
**tools** | Option<**std::collections::HashMap<String, bool>**> |  | [optional]
**enterprise** | Option<[**models::ConfigEnterprise**](Config_enterprise.md)> |  | [optional]
**compaction** | Option<[**models::ConfigCompaction**](Config_compaction.md)> |  | [optional]
**experimental** | Option<[**models::ConfigExperimental**](Config_experimental.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


