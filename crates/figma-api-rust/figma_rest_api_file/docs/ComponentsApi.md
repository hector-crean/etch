# \ComponentsApi

All URIs are relative to *https://api.figma.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_component**](ComponentsApi.md#get_component) | **GET** /v1/components/{key} | Get component



## get_component

> models::InlineObject14 get_component(key)
Get component

Get metadata on a component by key.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**key** | **String** | The unique identifier of the component. | [required] |

### Return type

[**models::InlineObject14**](inline_object_14.md)

### Authorization

[OAuth2](../README.md#OAuth2), [PersonalAccessToken](../README.md#PersonalAccessToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

