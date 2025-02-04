If you get a "Repository not found for url" error, you might be using a VPN.
If so, run the `. .\Get-HFToken.ps1` file to set the environment variable so HuggingFace trusts us.

```pwsh
❯ .\Invoke-Fulfillment.ps1
warning: `VIRTUAL_ENV=G:\repos\Nanuak\image-embedding-experiments\.venv` does not match the project environment path `G:\repos\Nanuak\.venv` and will be ignored
Launching...
Connected to Postgres
401 Client Error. (Request ID: Root=1-679e8edb-76a958d232c2fd1a473745c7;1841b646-5da2-43c5-874f-af16c1854a65)    

Repository Not Found for url: https://huggingface.co/openai/clip-vit-base-patch32/resolve/main/model.safetensors.
Please make sure you specified the correct `repo_id` and `repo_type`.
If you are trying to access a private or gated repo, make sure you are authenticated.
Invalid credentials in Authorization header
Traceback (most recent call last):
  File "G:\repos\Nanuak\.venv\lib\site-packages\huggingface_hub\utils\_http.py", line 406, in hf_raise_for_status
    response.raise_for_status()
  File "G:\repos\Nanuak\.venv\lib\site-packages\requests\models.py", line 1024, in raise_for_status
    raise HTTPError(http_error_msg, response=self)
requests.exceptions.HTTPError: 401 Client Error: Unauthorized for url: https://huggingface.co/openai/clip-vit-base-patch32/resolve/main/model.safetensors     

The above exception was the direct cause of the following exception:

Traceback (most recent call last):
  File "G:\repos\Nanuak\.venv\lib\site-packages\transformers\utils\hub.py", line 676, in has_file
    hf_raise_for_status(response)
  File "G:\repos\Nanuak\.venv\lib\site-packages\huggingface_hub\utils\_http.py", line 454, in hf_raise_for_status
    raise _format(RepositoryNotFoundError, message, response) from e
huggingface_hub.errors.RepositoryNotFoundError: 401 Client Error. (Request ID: Root=1-679e8edb-76a958d232c2fd1a473745c7;1841b646-5da2-43c5-874f-af16c1854a65) 

Repository Not Found for url: https://huggingface.co/openai/clip-vit-base-patch32/resolve/main/model.safetensors.
Please make sure you specified the correct `repo_id` and `repo_type`.
If you are trying to access a private or gated repo, make sure you are authenticated.
Invalid credentials in Authorization header

The above exception was the direct cause of the following exception:

Traceback (most recent call last):
  File "G:\repos\Nanuak\nanuak-files-index-fulfill\nanuak-files-index-fulfill.py", line 135, in <module>
    asyncio.run(main())
  File "C:\Users\TeamD\AppData\Roaming\uv\python\cpython-3.10.15-windows-x86_64-none\lib\asyncio\runners.py", line 44, in run
    return loop.run_until_complete(main)
  File "C:\Users\TeamD\AppData\Roaming\uv\python\cpython-3.10.15-windows-x86_64-none\lib\asyncio\base_events.py", line 649, in run_until_complete
    return future.result()
  File "G:\repos\Nanuak\nanuak-files-index-fulfill\nanuak-files-index-fulfill.py", line 31, in main
    clip_model = CLIPModel.from_pretrained(clip_model_name)
  File "G:\repos\Nanuak\.venv\lib\site-packages\transformers\modeling_utils.py", line 3891, in from_pretrained
    if not has_file(pretrained_model_name_or_path, safe_weights_name, **has_file_kwargs):
  File "G:\repos\Nanuak\.venv\lib\site-packages\transformers\utils\hub.py", line 687, in has_file
    raise EnvironmentError(
OSError: openai/clip-vit-base-patch32 is not a local folder or a valid repository name on 'https://hf.co'.
Nanuak\nanuak-files-index-fulfill on  main [✘!?] is 📦 v0.1.0 via 🐍 v3.10.15 (image-embedding-experiments) t
```