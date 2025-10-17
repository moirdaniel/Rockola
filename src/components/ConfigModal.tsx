import { useEffect, useState, type FormEvent } from 'react';

interface Props {
  dirs: string[];
  onSave: (value: string) => void;
  onClose: () => void;
}

export default function ConfigModal({ dirs, onSave, onClose }: Props) {
  const [value, setValue] = useState('');

  useEffect(() => {
    setValue(dirs.join(';'));
  }, [dirs]);

  function handleSubmit(e: FormEvent) {
    e.preventDefault();
    onSave(value);
    onClose();
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-6">
      <div className="card w-full max-w-xl p-6">
        <h2 className="mb-4 text-xl font-semibold">Configurar directorios de medios</h2>
        <p className="mb-4 text-sm text-muted">
          Ingresa rutas absolutas separadas por punto y coma (;). Se guardarán localmente y se usarán para futuras indexaciones.
        </p>
        <form className="grid gap-4" onSubmit={handleSubmit}>
          <textarea
            className="input h-32 resize-none"
            value={value}
            onChange={(e) => setValue(e.target.value)}
            placeholder="/home/usuario/Music;/home/usuario/Videos"
          />
          <div className="flex justify-end gap-3">
            <button type="button" className="btn" onClick={onClose}>Cancelar</button>
            <button type="submit" className="btn">Guardar</button>
          </div>
        </form>
      </div>
    </div>
  );
}
