import React, { useState } from 'react';
import { HttpAgent, Actor } from '@dfinity/agent';
import { idlFactory as storage_idl, canisterId as storage_canister_id } from 'declarations/storage_canister_backend'; // Adjust the import path

const FileUpload = () => {
  const [file, setFile] = useState(null);

  const handleFileChange = (event) => {
    setFile(event.target.files[0]);
  };

  const handleSubmit = async (event) => {
    event.preventDefault();
    if (!file) return;

    const reader = new FileReader();
    reader.onloadend = async () => {
      const arrayBuffer = reader.result;
      const bytes = new Uint8Array(arrayBuffer);

      const agent = new HttpAgent();
      await agent.fetchRootKey(); // For local development only. Remove this line when deploying to the live network.

      const actor = Actor.createActor(storage_idl, { agent, canisterId: storage_canister_id });

      try {
        await actor.upload_asset(file.name, Array.from(bytes));
        alert('File uploaded successfully');
      } catch (error) {
        console.error('Upload failed', error);
        alert('File upload failed');
      }
    };
    reader.readAsArrayBuffer(file);
  };

  return (
    <form onSubmit={handleSubmit}>
      <input type="file" onChange={handleFileChange} />
      <button type="submit">Upload</button>
    </form>
  );
};

export default FileUpload;
