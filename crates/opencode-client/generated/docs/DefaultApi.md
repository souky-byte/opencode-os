# \DefaultApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**app_agents**](DefaultApi.md#app_agents) | **GET** /agent | List agents
[**app_log**](DefaultApi.md#app_log) | **POST** /log | Write log
[**auth_set**](DefaultApi.md#auth_set) | **PUT** /auth/{providerID} | Set auth credentials
[**command_list**](DefaultApi.md#command_list) | **GET** /command | List commands
[**config_get**](DefaultApi.md#config_get) | **GET** /config | Get configuration
[**config_providers**](DefaultApi.md#config_providers) | **GET** /config/providers | List config providers
[**config_update**](DefaultApi.md#config_update) | **PATCH** /config | Update configuration
[**event_subscribe**](DefaultApi.md#event_subscribe) | **GET** /event | Subscribe to events
[**file_list**](DefaultApi.md#file_list) | **GET** /file | List files
[**file_read**](DefaultApi.md#file_read) | **GET** /file/content | Read file
[**file_status**](DefaultApi.md#file_status) | **GET** /file/status | Get file status
[**find_files**](DefaultApi.md#find_files) | **GET** /find/file | Find files
[**find_symbols**](DefaultApi.md#find_symbols) | **GET** /find/symbol | Find symbols
[**find_text**](DefaultApi.md#find_text) | **GET** /find | Find text
[**formatter_status**](DefaultApi.md#formatter_status) | **GET** /formatter | Get formatter status
[**global_dispose**](DefaultApi.md#global_dispose) | **POST** /global/dispose | Dispose instance
[**global_event**](DefaultApi.md#global_event) | **GET** /global/event | Get global events
[**global_health**](DefaultApi.md#global_health) | **GET** /global/health | Get health
[**instance_dispose**](DefaultApi.md#instance_dispose) | **POST** /instance/dispose | Dispose instance
[**lsp_status**](DefaultApi.md#lsp_status) | **GET** /lsp | Get LSP status
[**mcp_add**](DefaultApi.md#mcp_add) | **POST** /mcp | Add MCP server
[**mcp_auth_authenticate**](DefaultApi.md#mcp_auth_authenticate) | **POST** /mcp/{name}/auth/authenticate | Authenticate MCP OAuth
[**mcp_auth_callback**](DefaultApi.md#mcp_auth_callback) | **POST** /mcp/{name}/auth/callback | Complete MCP OAuth
[**mcp_auth_remove**](DefaultApi.md#mcp_auth_remove) | **DELETE** /mcp/{name}/auth | Remove MCP OAuth
[**mcp_auth_start**](DefaultApi.md#mcp_auth_start) | **POST** /mcp/{name}/auth | Start MCP OAuth
[**mcp_connect**](DefaultApi.md#mcp_connect) | **POST** /mcp/{name}/connect | 
[**mcp_disconnect**](DefaultApi.md#mcp_disconnect) | **POST** /mcp/{name}/disconnect | 
[**mcp_status**](DefaultApi.md#mcp_status) | **GET** /mcp | Get MCP status
[**part_delete**](DefaultApi.md#part_delete) | **DELETE** /session/{sessionID}/message/{messageID}/part/{partID} | 
[**part_update**](DefaultApi.md#part_update) | **PATCH** /session/{sessionID}/message/{messageID}/part/{partID} | 
[**path_get**](DefaultApi.md#path_get) | **GET** /path | Get paths
[**permission_list**](DefaultApi.md#permission_list) | **GET** /permission | List pending permissions
[**permission_respond**](DefaultApi.md#permission_respond) | **POST** /session/{sessionID}/permissions/{permissionID} | Respond to permission
[**project_current**](DefaultApi.md#project_current) | **GET** /project/current | Get current project
[**project_list**](DefaultApi.md#project_list) | **GET** /project | List all projects
[**project_update**](DefaultApi.md#project_update) | **PATCH** /project/{projectID} | Update project
[**provider_auth**](DefaultApi.md#provider_auth) | **GET** /provider/auth | Get provider auth methods
[**provider_list**](DefaultApi.md#provider_list) | **GET** /provider | List providers
[**provider_oauth_authorize**](DefaultApi.md#provider_oauth_authorize) | **POST** /provider/{providerID}/oauth/authorize | OAuth authorize
[**provider_oauth_callback**](DefaultApi.md#provider_oauth_callback) | **POST** /provider/{providerID}/oauth/callback | OAuth callback
[**pty_connect**](DefaultApi.md#pty_connect) | **GET** /pty/{ptyID}/connect | Connect to PTY session
[**pty_create**](DefaultApi.md#pty_create) | **POST** /pty | Create PTY session
[**pty_get**](DefaultApi.md#pty_get) | **GET** /pty/{ptyID} | Get PTY session
[**pty_list**](DefaultApi.md#pty_list) | **GET** /pty | List PTY sessions
[**pty_remove**](DefaultApi.md#pty_remove) | **DELETE** /pty/{ptyID} | Remove PTY session
[**pty_update**](DefaultApi.md#pty_update) | **PUT** /pty/{ptyID} | Update PTY session
[**session_abort**](DefaultApi.md#session_abort) | **POST** /session/{sessionID}/abort | Abort session
[**session_command**](DefaultApi.md#session_command) | **POST** /session/{sessionID}/command | Send command
[**session_create**](DefaultApi.md#session_create) | **POST** /session | Create session
[**session_delete**](DefaultApi.md#session_delete) | **DELETE** /session/{sessionID} | Delete session
[**session_diff**](DefaultApi.md#session_diff) | **GET** /session/{sessionID}/diff | Get session diff
[**session_fork**](DefaultApi.md#session_fork) | **POST** /session/{sessionID}/fork | Fork session
[**session_init**](DefaultApi.md#session_init) | **POST** /session/{sessionID}/init | Initialize session
[**session_list**](DefaultApi.md#session_list) | **GET** /session | List sessions
[**session_message**](DefaultApi.md#session_message) | **GET** /session/{sessionID}/message/{messageID} | Get message
[**session_messages**](DefaultApi.md#session_messages) | **GET** /session/{sessionID}/message | Get session messages
[**session_prompt**](DefaultApi.md#session_prompt) | **POST** /session/{sessionID}/message | Send message
[**session_prompt_async**](DefaultApi.md#session_prompt_async) | **POST** /session/{sessionID}/prompt_async | Send async message
[**session_revert**](DefaultApi.md#session_revert) | **POST** /session/{sessionID}/revert | Revert message
[**session_share**](DefaultApi.md#session_share) | **POST** /session/{sessionID}/share | Share session
[**session_shell**](DefaultApi.md#session_shell) | **POST** /session/{sessionID}/shell | Run shell command
[**session_status**](DefaultApi.md#session_status) | **GET** /session/status | Get session status
[**session_summarize**](DefaultApi.md#session_summarize) | **POST** /session/{sessionID}/summarize | Summarize session
[**session_todo**](DefaultApi.md#session_todo) | **GET** /session/{sessionID}/todo | Get session todos
[**session_unrevert**](DefaultApi.md#session_unrevert) | **POST** /session/{sessionID}/unrevert | Restore reverted messages
[**session_unshare**](DefaultApi.md#session_unshare) | **DELETE** /session/{sessionID}/share | Unshare session
[**session_update**](DefaultApi.md#session_update) | **PATCH** /session/{sessionID} | Update session
[**tool_ids**](DefaultApi.md#tool_ids) | **GET** /experimental/tool/ids | List tool IDs
[**tool_list**](DefaultApi.md#tool_list) | **GET** /experimental/tool | List tools
[**tui_append_prompt**](DefaultApi.md#tui_append_prompt) | **POST** /tui/append-prompt | Append TUI prompt
[**tui_clear_prompt**](DefaultApi.md#tui_clear_prompt) | **POST** /tui/clear-prompt | Clear TUI prompt
[**tui_control_next**](DefaultApi.md#tui_control_next) | **GET** /tui/control/next | Get next TUI request
[**tui_control_response**](DefaultApi.md#tui_control_response) | **POST** /tui/control/response | Submit TUI response
[**tui_execute_command**](DefaultApi.md#tui_execute_command) | **POST** /tui/execute-command | Execute TUI command
[**tui_open_help**](DefaultApi.md#tui_open_help) | **POST** /tui/open-help | Open help dialog
[**tui_open_models**](DefaultApi.md#tui_open_models) | **POST** /tui/open-models | Open models dialog
[**tui_open_sessions**](DefaultApi.md#tui_open_sessions) | **POST** /tui/open-sessions | Open sessions dialog
[**tui_open_themes**](DefaultApi.md#tui_open_themes) | **POST** /tui/open-themes | Open themes dialog
[**tui_publish**](DefaultApi.md#tui_publish) | **POST** /tui/publish | Publish TUI event
[**tui_show_toast**](DefaultApi.md#tui_show_toast) | **POST** /tui/show-toast | Show TUI toast
[**tui_submit_prompt**](DefaultApi.md#tui_submit_prompt) | **POST** /tui/submit-prompt | Submit TUI prompt
[**vcs_get**](DefaultApi.md#vcs_get) | **GET** /vcs | Get VCS info



## app_agents

> Vec<models::Agent> app_agents(directory)
List agents

Get a list of all available AI agents in the OpenCode system.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**Vec<models::Agent>**](Agent.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## app_log

> bool app_log(directory, app_log_request)
Write log

Write a log entry to the server logs with specified level and metadata.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |
**app_log_request** | Option<[**AppLogRequest**](AppLogRequest.md)> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_set

> bool auth_set(provider_id, directory, auth)
Set auth credentials

Set authentication credentials

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**provider_id** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |
**auth** | Option<[**Auth**](Auth.md)> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## command_list

> Vec<models::Command> command_list(directory)
List commands

Get a list of all available commands in the OpenCode system.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**Vec<models::Command>**](Command.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## config_get

> models::Config config_get(directory)
Get configuration

Retrieve the current OpenCode configuration settings and preferences.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**models::Config**](Config.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## config_providers

> models::ConfigProviders200Response config_providers(directory)
List config providers

Get a list of all configured AI providers and their default models.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**models::ConfigProviders200Response**](config_providers_200_response.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## config_update

> models::Config config_update(directory, config)
Update configuration

Update OpenCode configuration settings and preferences.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |
**config** | Option<[**Config**](Config.md)> |  |  |

### Return type

[**models::Config**](Config.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## event_subscribe

> models::Event event_subscribe(directory)
Subscribe to events

Get events

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**models::Event**](Event.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/event-stream

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## file_list

> Vec<models::FileNode> file_list(path, directory)
List files

List files and directories in a specified path.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**path** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

[**Vec<models::FileNode>**](FileNode.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## file_read

> models::FileContent file_read(path, directory)
Read file

Read the content of a specified file.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**path** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

[**models::FileContent**](FileContent.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## file_status

> Vec<models::File> file_status(directory)
Get file status

Get the git status of all files in the project.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**Vec<models::File>**](File.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## find_files

> Vec<String> find_files(query, directory, dirs, r#type, limit)
Find files

Search for files or directories by name or pattern in the project directory.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**query** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |
**dirs** | Option<**String**> |  |  |
**r#type** | Option<**String**> |  |  |
**limit** | Option<**i32**> |  |  |

### Return type

**Vec<String>**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## find_symbols

> Vec<models::Symbol> find_symbols(query, directory)
Find symbols

Search for workspace symbols like functions, classes, and variables using LSP.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**query** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

[**Vec<models::Symbol>**](Symbol.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## find_text

> Vec<models::FindText200ResponseInner> find_text(pattern, directory)
Find text

Search for text patterns across files in the project using ripgrep.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**pattern** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

[**Vec<models::FindText200ResponseInner>**](find_text_200_response_inner.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## formatter_status

> Vec<models::FormatterStatus> formatter_status(directory)
Get formatter status

Get formatter status

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**Vec<models::FormatterStatus>**](FormatterStatus.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## global_dispose

> bool global_dispose()
Dispose instance

Clean up and dispose all OpenCode instances, releasing all resources.

### Parameters

This endpoint does not need any parameter.

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## global_event

> models::GlobalEvent global_event()
Get global events

Subscribe to global events from the OpenCode system using server-sent events.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GlobalEvent**](GlobalEvent.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/event-stream

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## global_health

> models::GlobalHealth200Response global_health()
Get health

Get health information about the OpenCode server.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GlobalHealth200Response**](global_health_200_response.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## instance_dispose

> bool instance_dispose(directory)
Dispose instance

Clean up and dispose the current OpenCode instance, releasing all resources.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## lsp_status

> Vec<models::LspStatus> lsp_status(directory)
Get LSP status

Get LSP server status

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**Vec<models::LspStatus>**](LSPStatus.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## mcp_add

> std::collections::HashMap<String, models::McpStatus> mcp_add(directory, mcp_add_request)
Add MCP server

Dynamically add a new Model Context Protocol (MCP) server to the system.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |
**mcp_add_request** | Option<[**McpAddRequest**](McpAddRequest.md)> |  |  |

### Return type

[**std::collections::HashMap<String, models::McpStatus>**](MCPStatus.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## mcp_auth_authenticate

> models::McpStatus mcp_auth_authenticate(name, directory)
Authenticate MCP OAuth

Start OAuth flow and wait for callback (opens browser)

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

[**models::McpStatus**](MCPStatus.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## mcp_auth_callback

> models::McpStatus mcp_auth_callback(name, directory, mcp_auth_callback_request)
Complete MCP OAuth

Complete OAuth authentication for a Model Context Protocol (MCP) server using the authorization code.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |
**mcp_auth_callback_request** | Option<[**McpAuthCallbackRequest**](McpAuthCallbackRequest.md)> |  |  |

### Return type

[**models::McpStatus**](MCPStatus.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## mcp_auth_remove

> models::McpAuthRemove200Response mcp_auth_remove(name, directory)
Remove MCP OAuth

Remove OAuth credentials for an MCP server

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

[**models::McpAuthRemove200Response**](mcp_auth_remove_200_response.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## mcp_auth_start

> models::McpAuthStart200Response mcp_auth_start(name, directory)
Start MCP OAuth

Start OAuth authentication flow for a Model Context Protocol (MCP) server.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

[**models::McpAuthStart200Response**](mcp_auth_start_200_response.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## mcp_connect

> bool mcp_connect(name, directory)


Connect an MCP server

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## mcp_disconnect

> bool mcp_disconnect(name, directory)


Disconnect an MCP server

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## mcp_status

> std::collections::HashMap<String, models::McpStatus> mcp_status(directory)
Get MCP status

Get the status of all Model Context Protocol (MCP) servers.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**std::collections::HashMap<String, models::McpStatus>**](MCPStatus.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## part_delete

> bool part_delete(session_id, message_id, part_id, directory)


Delete a part from a message

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** | Session ID | [required] |
**message_id** | **String** | Message ID | [required] |
**part_id** | **String** | Part ID | [required] |
**directory** | Option<**String**> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## part_update

> models::Part part_update(session_id, message_id, part_id, directory, part)


Update a part in a message

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** | Session ID | [required] |
**message_id** | **String** | Message ID | [required] |
**part_id** | **String** | Part ID | [required] |
**directory** | Option<**String**> |  |  |
**part** | Option<[**Part**](Part.md)> |  |  |

### Return type

[**models::Part**](Part.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## path_get

> models::Path path_get(directory)
Get paths

Retrieve the current working directory and related path information for the OpenCode instance.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**models::Path**](Path.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## permission_list

> Vec<models::Permission> permission_list(directory)
List pending permissions

Get all pending permission requests across all sessions.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**Vec<models::Permission>**](Permission.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## permission_respond

> bool permission_respond(session_id, permission_id, directory, permission_respond_request)
Respond to permission

Approve or deny a permission request from the AI assistant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** |  | [required] |
**permission_id** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |
**permission_respond_request** | Option<[**PermissionRespondRequest**](PermissionRespondRequest.md)> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## project_current

> models::Project project_current(directory)
Get current project

Retrieve the currently active project that OpenCode is working with.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**models::Project**](Project.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## project_list

> Vec<models::Project> project_list(directory)
List all projects

Get a list of projects that have been opened with OpenCode.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**Vec<models::Project>**](Project.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## project_update

> models::Project project_update(project_id, directory, project_update_request)
Update project

Update project properties such as name, icon and color.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**project_id** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |
**project_update_request** | Option<[**ProjectUpdateRequest**](ProjectUpdateRequest.md)> |  |  |

### Return type

[**models::Project**](Project.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## provider_auth

> std::collections::HashMap<String, Vec<models::ProviderAuthMethod>> provider_auth(directory)
Get provider auth methods

Retrieve available authentication methods for all AI providers.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**std::collections::HashMap<String, Vec<models::ProviderAuthMethod>>**](Vec.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## provider_list

> models::ProviderList200Response provider_list(directory)
List providers

Get a list of all available AI providers, including both available and connected ones.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**models::ProviderList200Response**](provider_list_200_response.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## provider_oauth_authorize

> models::ProviderAuthAuthorization provider_oauth_authorize(provider_id, directory, provider_oauth_authorize_request)
OAuth authorize

Initiate OAuth authorization for a specific AI provider to get an authorization URL.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**provider_id** | **String** | Provider ID | [required] |
**directory** | Option<**String**> |  |  |
**provider_oauth_authorize_request** | Option<[**ProviderOauthAuthorizeRequest**](ProviderOauthAuthorizeRequest.md)> |  |  |

### Return type

[**models::ProviderAuthAuthorization**](ProviderAuthAuthorization.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## provider_oauth_callback

> bool provider_oauth_callback(provider_id, directory, provider_oauth_callback_request)
OAuth callback

Handle the OAuth callback from a provider after user authorization.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**provider_id** | **String** | Provider ID | [required] |
**directory** | Option<**String**> |  |  |
**provider_oauth_callback_request** | Option<[**ProviderOauthCallbackRequest**](ProviderOauthCallbackRequest.md)> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## pty_connect

> bool pty_connect(pty_id, directory)
Connect to PTY session

Establish a WebSocket connection to interact with a pseudo-terminal (PTY) session in real-time.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**pty_id** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## pty_create

> models::Pty pty_create(directory, pty_create_request)
Create PTY session

Create a new pseudo-terminal (PTY) session for running shell commands and processes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |
**pty_create_request** | Option<[**PtyCreateRequest**](PtyCreateRequest.md)> |  |  |

### Return type

[**models::Pty**](Pty.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## pty_get

> models::Pty pty_get(pty_id, directory)
Get PTY session

Retrieve detailed information about a specific pseudo-terminal (PTY) session.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**pty_id** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

[**models::Pty**](Pty.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## pty_list

> Vec<models::Pty> pty_list(directory)
List PTY sessions

Get a list of all active pseudo-terminal (PTY) sessions managed by OpenCode.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**Vec<models::Pty>**](Pty.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## pty_remove

> bool pty_remove(pty_id, directory)
Remove PTY session

Remove and terminate a specific pseudo-terminal (PTY) session.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**pty_id** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## pty_update

> models::Pty pty_update(pty_id, directory, pty_update_request)
Update PTY session

Update properties of an existing pseudo-terminal (PTY) session.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**pty_id** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |
**pty_update_request** | Option<[**PtyUpdateRequest**](PtyUpdateRequest.md)> |  |  |

### Return type

[**models::Pty**](Pty.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_abort

> bool session_abort(session_id, directory)
Abort session

Abort an active session and stop any ongoing AI processing or command execution.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_command

> models::SessionPrompt200Response session_command(session_id, directory, session_command_request)
Send command

Send a new command to a session for execution by the AI assistant.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** | Session ID | [required] |
**directory** | Option<**String**> |  |  |
**session_command_request** | Option<[**SessionCommandRequest**](SessionCommandRequest.md)> |  |  |

### Return type

[**models::SessionPrompt200Response**](session_prompt_200_response.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_create

> models::Session session_create(directory, session_create_request)
Create session

Create a new OpenCode session for interacting with AI assistants and managing conversations.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |
**session_create_request** | Option<[**SessionCreateRequest**](SessionCreateRequest.md)> |  |  |

### Return type

[**models::Session**](Session.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_delete

> bool session_delete(session_id, directory)
Delete session

Delete a session and permanently remove all associated data, including messages and history.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_diff

> Vec<models::FileDiff> session_diff(session_id, directory, message_id)
Get session diff

Get all file changes (diffs) made during this session.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** | Session ID | [required] |
**directory** | Option<**String**> |  |  |
**message_id** | Option<**String**> |  |  |

### Return type

[**Vec<models::FileDiff>**](FileDiff.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_fork

> models::Session session_fork(session_id, directory, session_fork_request)
Fork session

Create a new session by forking an existing session at a specific message point.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |
**session_fork_request** | Option<[**SessionForkRequest**](SessionForkRequest.md)> |  |  |

### Return type

[**models::Session**](Session.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_init

> bool session_init(session_id, directory, session_init_request)
Initialize session

Analyze the current application and create an AGENTS.md file with project-specific agent configurations.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** | Session ID | [required] |
**directory** | Option<**String**> |  |  |
**session_init_request** | Option<[**SessionInitRequest**](SessionInitRequest.md)> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_list

> Vec<models::Session> session_list(directory)
List sessions

Get a list of all OpenCode sessions, sorted by most recently updated.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**Vec<models::Session>**](Session.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_message

> models::SessionMessages200ResponseInner session_message(session_id, message_id, directory)
Get message

Retrieve a specific message from a session by its message ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** | Session ID | [required] |
**message_id** | **String** | Message ID | [required] |
**directory** | Option<**String**> |  |  |

### Return type

[**models::SessionMessages200ResponseInner**](session_messages_200_response_inner.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_messages

> Vec<models::SessionMessages200ResponseInner> session_messages(session_id, directory, limit)
Get session messages

Retrieve all messages in a session, including user prompts and AI responses.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** | Session ID | [required] |
**directory** | Option<**String**> |  |  |
**limit** | Option<**f64**> |  |  |

### Return type

[**Vec<models::SessionMessages200ResponseInner>**](session_messages_200_response_inner.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_prompt

> models::SessionPrompt200Response session_prompt(session_id, directory, session_prompt_request)
Send message

Create and send a new message to a session, streaming the AI response.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** | Session ID | [required] |
**directory** | Option<**String**> |  |  |
**session_prompt_request** | Option<[**SessionPromptRequest**](SessionPromptRequest.md)> |  |  |

### Return type

[**models::SessionPrompt200Response**](session_prompt_200_response.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_prompt_async

> session_prompt_async(session_id, directory, session_prompt_request)
Send async message

Create and send a new message to a session asynchronously, starting the session if needed and returning immediately.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** | Session ID | [required] |
**directory** | Option<**String**> |  |  |
**session_prompt_request** | Option<[**SessionPromptRequest**](SessionPromptRequest.md)> |  |  |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_revert

> models::Session session_revert(session_id, directory, session_revert_request)
Revert message

Revert a specific message in a session, undoing its effects and restoring the previous state.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |
**session_revert_request** | Option<[**SessionRevertRequest**](SessionRevertRequest.md)> |  |  |

### Return type

[**models::Session**](Session.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_share

> models::Session session_share(session_id, directory)
Share session

Create a shareable link for a session, allowing others to view the conversation.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

[**models::Session**](Session.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_shell

> models::AssistantMessage session_shell(session_id, directory, session_shell_request)
Run shell command

Execute a shell command within the session context and return the AI's response.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** | Session ID | [required] |
**directory** | Option<**String**> |  |  |
**session_shell_request** | Option<[**SessionShellRequest**](SessionShellRequest.md)> |  |  |

### Return type

[**models::AssistantMessage**](AssistantMessage.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_status

> std::collections::HashMap<String, models::SessionStatus> session_status(directory)
Get session status

Retrieve the current status of all sessions, including active, idle, and completed states.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**std::collections::HashMap<String, models::SessionStatus>**](SessionStatus.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_summarize

> bool session_summarize(session_id, directory, session_summarize_request)
Summarize session

Generate a concise summary of the session using AI compaction to preserve key information.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** | Session ID | [required] |
**directory** | Option<**String**> |  |  |
**session_summarize_request** | Option<[**SessionSummarizeRequest**](SessionSummarizeRequest.md)> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_todo

> Vec<models::Todo> session_todo(session_id, directory)
Get session todos

Retrieve the todo list associated with a specific session, showing tasks and action items.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** | Session ID | [required] |
**directory** | Option<**String**> |  |  |

### Return type

[**Vec<models::Todo>**](Todo.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_unrevert

> models::Session session_unrevert(session_id, directory)
Restore reverted messages

Restore all previously reverted messages in a session.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

[**models::Session**](Session.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_unshare

> models::Session session_unshare(session_id, directory)
Unshare session

Remove the shareable link for a session, making it private again.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

[**models::Session**](Session.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_update

> models::Session session_update(session_id, directory, session_update_request)
Update session

Update properties of an existing session, such as title or other metadata.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |
**session_update_request** | Option<[**SessionUpdateRequest**](SessionUpdateRequest.md)> |  |  |

### Return type

[**models::Session**](Session.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tool_ids

> Vec<String> tool_ids(directory)
List tool IDs

Get a list of all available tool IDs, including both built-in tools and dynamically registered tools.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

**Vec<String>**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tool_list

> Vec<models::ToolListItem> tool_list(provider, model, directory)
List tools

Get a list of available tools with their JSON schema parameters for a specific provider and model combination.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**provider** | **String** |  | [required] |
**model** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

[**Vec<models::ToolListItem>**](ToolListItem.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tui_append_prompt

> bool tui_append_prompt(directory, find_text200_response_inner_path)
Append TUI prompt

Append prompt to the TUI

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |
**find_text200_response_inner_path** | Option<[**FindText200ResponseInnerPath**](FindText200ResponseInnerPath.md)> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tui_clear_prompt

> bool tui_clear_prompt(directory)
Clear TUI prompt

Clear the prompt

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tui_control_next

> models::TuiControlNext200Response tui_control_next(directory)
Get next TUI request

Retrieve the next TUI (Terminal User Interface) request from the queue for processing.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**models::TuiControlNext200Response**](tui_control_next_200_response.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tui_control_response

> bool tui_control_response(directory, body)
Submit TUI response

Submit a response to the TUI request queue to complete a pending request.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |
**body** | Option<**serde_json::Value**> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tui_execute_command

> bool tui_execute_command(directory, tui_execute_command_request)
Execute TUI command

Execute a TUI command (e.g. agent_cycle)

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |
**tui_execute_command_request** | Option<[**TuiExecuteCommandRequest**](TuiExecuteCommandRequest.md)> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tui_open_help

> bool tui_open_help(directory)
Open help dialog

Open the help dialog in the TUI to display user assistance information.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tui_open_models

> bool tui_open_models(directory)
Open models dialog

Open the model dialog

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tui_open_sessions

> bool tui_open_sessions(directory)
Open sessions dialog

Open the session dialog

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tui_open_themes

> bool tui_open_themes(directory)
Open themes dialog

Open the theme dialog

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tui_publish

> bool tui_publish(directory, tui_publish_request)
Publish TUI event

Publish a TUI event

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |
**tui_publish_request** | Option<[**TuiPublishRequest**](TuiPublishRequest.md)> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tui_show_toast

> bool tui_show_toast(directory, tui_show_toast_request)
Show TUI toast

Show a toast notification in the TUI

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |
**tui_show_toast_request** | Option<[**TuiShowToastRequest**](TuiShowToastRequest.md)> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tui_submit_prompt

> bool tui_submit_prompt(directory)
Submit TUI prompt

Submit the prompt

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

**bool**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## vcs_get

> models::VcsInfo vcs_get(directory)
Get VCS info

Retrieve version control system (VCS) information for the current project, such as git branch.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**directory** | Option<**String**> |  |  |

### Return type

[**models::VcsInfo**](VcsInfo.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

