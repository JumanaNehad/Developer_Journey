import React, { useState } from 'react';
import { HttpAgent, Actor } from '@dfinity/agent';
import { idlFactory, canisterId } from 'declarations/storage_canister_backend'; // Replace with the correct import path

const CHUNK_SIZE = 1.9 * 1024 * 1024; // 1.9MB chunk size to accommodate overhead

const FileUpload = () => {
  const [file, setFile] = useState(null);
  const [uploading, setUploading] = useState(false);

  const handleFileChange = (event) => {
    setFile(event.target.files[0]);
  };

  const handleSubmit = async (event) => {
    event.preventDefault();
    if (!file) return;

    setUploading(true);
    const totalChunks = Math.ceil(file.size / CHUNK_SIZE);

    for (let i = 0; i < totalChunks; i++) {
      const start = i * CHUNK_SIZE;
      const end = Math.min(start + CHUNK_SIZE, file.size);
      const chunk = file.slice(start, end);

      try {
        await uploadChunk(chunk, i);
        console.log(`Uploaded chunk ${i}`);
      } catch (error) {
        console.error(`Upload failed for chunk ${i}`, error);
        alert(`File upload failed at chunk ${i}`);
        setUploading(false);
        return;
      }
    }

    setUploading(false);
    alert('File uploaded successfully');
  };

  const uploadChunk = async (chunk, order) => {
    const reader = new FileReader();
    reader.onloadend = async () => {
      const arrayBuffer = reader.result;
      const bytes = new Uint8Array(arrayBuffer);

      const agent = new HttpAgent();
      await agent.fetchRootKey(); // For local development only, remove in production

      const actor = Actor.createActor(idlFactory, { agent, canisterId });

      try {
        await actor.upload_chunk({ order, content: Array.from(bytes) });
      } catch (error) {
        throw new Error(`Failed to upload chunk ${order}: ${error}`);
      }
    };
    reader.readAsArrayBuffer(chunk);
  };

  return (
    <form onSubmit={handleSubmit}>
      <input type="file" onChange={handleFileChange} />
      <button type="submit" disabled={uploading}>
        {uploading ? 'Uploading...' : 'Upload'}
      </button>
    </form>
  );
};

export default FileUpload;
