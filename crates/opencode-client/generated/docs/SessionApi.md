# \SessionApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**session_children**](SessionApi.md#session_children) | **GET** /session/{sessionID}/children | Get session children
[**session_get**](SessionApi.md#session_get) | **GET** /session/{sessionID} | Get session



## session_children

> Vec<models::Session> session_children(session_id, directory)
Get session children

Retrieve all child sessions that were forked from the specified parent session.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**session_id** | **String** |  | [required] |
**directory** | Option<**String**> |  |  |

### Return type

[**Vec<models::Session>**](Session.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## session_get

> models::Session session_get(session_id, directory)
Get session

Retrieve detailed information about a specific OpenCode session.

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

