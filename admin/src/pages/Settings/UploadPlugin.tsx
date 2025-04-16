import { Box, Button, FileInput, Flex } from "@mantine/core";
import { readFileAsString } from "@/utils/file-utils";
import { PlusCircle } from "lucide-react";
import { Plugin, PluginManifest } from "@/types";
import { useState } from "react";
import { useForm } from "@mantine/form";
import { createPlugin } from "@/api/plugins";

export default function UploadPlugin() {
  const [error, setError] = useState<Error|null>(null);
  const [manifest, setManifest] = useState<PluginManifest|null>(null);
  const form = useForm({
    initialValues: {
      id: 'new',
      name: '',
      version: '',
    },
    validate: {
      name: (value) => value ? null : 'Name is required',
      version: (value) => value ? null : 'Version is required',
    },
  });

  async function loadManifest(f: File) {
    const data = await readFileAsString(f);
    const manifest = JSON.parse(data) as PluginManifest;
    console.log(manifest);
    form.setValues(manifest);
    form.validate();
    setManifest(manifest as PluginManifest);
  }

  async function handleSubmit(values: typeof form.values) {
    const resp = await createPlugin({...values, manifest, id: 'new'} as Plugin);
    console.warn(resp);
  }

  return (<form onSubmit={form.onSubmit(handleSubmit)}>
    <Flex gap="md">
      <FileInput flex={1} label="manifest" description="Manifest" accept="application/json" onChange={loadManifest}/>
      <FileInput flex={1} label="wasm" description="WASM" accept="*.wasm"/>
    </Flex>
    <Flex justify="end" mt="md">
      <Button leftSection={<PlusCircle />} disabled={!form.isValid()} type="submit">
        <input type="file" style={{display: 'none'}} />
        Upload
      </Button>
    </Flex>
  </form>);
}
