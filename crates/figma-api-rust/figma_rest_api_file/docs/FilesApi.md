# \FilesApi

All URIs are relative to *https://api.figma.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_file**](FilesApi.md#get_file) | **GET** /v1/files/{file_key} | Get file JSON
[**get_file_nodes**](FilesApi.md#get_file_nodes) | **GET** /v1/files/{file_key}/nodes | Get file JSON for specific nodes
[**get_image_fills**](FilesApi.md#get_image_fills) | **GET** /v1/files/{file_key}/images | Get image fills



## get_file

> models::InlineObject get_file(file_key, version, ids, depth, geometry, plugin_data, branch_data)
Get file JSON

Returns the document identified by `file_key` as a JSON object. The file key can be parsed from any Figma file url: `https://www.figma.com/file/{file_key}/{title}`.  The `document` property contains a node of type `DOCUMENT`.  The `components` property contains a mapping from node IDs to component metadata. This is to help you determine which components each instance comes from.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**file_key** | **String** | File to export JSON from. This can be a file key or branch key. Use `GET /v1/files/:key` with the `branch_data` query param to get the branch key. | [required] |
**version** | Option<**String**> | A specific version ID to get. Omitting this will get the current version of the file. |  |
**ids** | Option<**String**> | Comma separated list of nodes that you care about in the document. If specified, only a subset of the document will be returned corresponding to the nodes listed, their children, and everything between the root node and the listed nodes.  Note: There may be other nodes included in the returned JSON that are outside the ancestor chains of the desired nodes. The response may also include dependencies of anything in the nodes' subtrees. For example, if a node subtree contains an instance of a local component that lives elsewhere in that file, that component and its ancestor chain will also be included.  For historical reasons, top-level canvas nodes are always returned, regardless of whether they are listed in the `ids` parameter. This quirk may be removed in a future version of the API. |  |
**depth** | Option<**f64**> | Positive integer representing how deep into the document tree to traverse. For example, setting this to 1 returns only Pages, setting it to 2 returns Pages and all top level objects on each page. Not setting this parameter returns all nodes. |  |
**geometry** | Option<**String**> | Set to \"paths\" to export vector data. |  |
**plugin_data** | Option<**String**> | A comma separated list of plugin IDs and/or the string \"shared\". Any data present in the document written by those plugins will be included in the result in the `pluginData` and `sharedPluginData` properties. |  |
**branch_data** | Option<**bool**> | Returns branch metadata for the requested file. If the file is a branch, the main file's key will be included in the returned response. If the file has branches, their metadata will be included in the returned response. Default: false. |  |[default to false]

### Return type

[**models::InlineObject**](inline_object.md)

### Authorization

[OAuth2](../README.md#OAuth2), [PersonalAccessToken](../README.md#PersonalAccessToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_file_nodes

> models::InlineObject1 get_file_nodes(file_key, ids, version, depth, geometry, plugin_data)
Get file JSON for specific nodes

Returns the nodes referenced to by `ids` as a JSON object. The nodes are retrieved from the Figma file referenced to by `file_key`.  The node ID and file key can be parsed from any Figma node url: `https://www.figma.com/file/{file_key}/{title}?node-id={id}`  The `name`, `lastModified`, `thumbnailUrl`, `editorType`, and `version` attributes are all metadata of the specified file.  The `linkAccess` field describes the file link share permission level. There are 5 types of permissions a shared link can have: `\"inherit\"`, `\"view\"`, `\"edit\"`, `\"org_view\"`, and `\"org_edit\"`. `\"inherit\"` is the default permission applied to files created in a team project, and will inherit the project's permissions. `\"org_view\"` and `\"org_edit\"` restrict the link to org users.  The `document` attribute contains a Node of type `DOCUMENT`.  The `components` key contains a mapping from node IDs to component metadata. This is to help you determine which components each instance comes from.  By default, no vector data is returned. To return vector data, pass the geometry=paths parameter to the endpoint. Each node can also inherit properties from applicable styles. The styles key contains a mapping from style IDs to style metadata.  Important: the nodes map may contain values that are `null`. This may be due to the node id not existing within the specified file.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**file_key** | **String** | File to export JSON from. This can be a file key or branch key. Use `GET /v1/files/:key` with the `branch_data` query param to get the branch key. | [required] |
**ids** | **String** | A comma separated list of node IDs to retrieve and convert. | [required] |
**version** | Option<**String**> | A specific version ID to get. Omitting this will get the current version of the file. |  |
**depth** | Option<**f64**> | Positive integer representing how deep into the node tree to traverse. For example, setting this to 1 will return only the children directly underneath the desired nodes. Not setting this parameter returns all nodes.  Note: this parameter behaves differently from the same parameter in the `GET /v1/files/:key` endpoint. In this endpoint, the depth will be counted starting from the desired node rather than the document root node. |  |
**geometry** | Option<**String**> | Set to \"paths\" to export vector data. |  |
**plugin_data** | Option<**String**> | A comma separated list of plugin IDs and/or the string \"shared\". Any data present in the document written by those plugins will be included in the result in the `pluginData` and `sharedPluginData` properties. |  |

### Return type

[**models::InlineObject1**](inline_object_1.md)

### Authorization

[OAuth2](../README.md#OAuth2), [PersonalAccessToken](../README.md#PersonalAccessToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_image_fills

> models::InlineObject3 get_image_fills(file_key)
Get image fills

Returns download links for all images present in image fills in a document. Image fills are how Figma represents any user supplied images. When you drag an image into Figma, we create a rectangle with a single fill that represents the image, and the user is able to transform the rectangle (and properties on the fill) as they wish.  This endpoint returns a mapping from image references to the URLs at which the images may be download. Image URLs will expire after no more than 14 days. Image references are located in the output of the GET files endpoint under the `imageRef` attribute in a `Paint`.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**file_key** | **String** | File to get image URLs from. This can be a file key or branch key. Use `GET /v1/files/:key` with the `branch_data` query param to get the branch key. | [required] |

### Return type

[**models::InlineObject3**](inline_object_3.md)

### Authorization

[OAuth2](../README.md#OAuth2), [PersonalAccessToken](../README.md#PersonalAccessToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

