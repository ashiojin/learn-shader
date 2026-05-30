bl_info = {
    "name": "Ashiojin Engine Material Tools",
    "author": "Ashiojin",
    "version": (1, 0),
    "blender": (4, 0, 0),  # Adjust this to your current Blender version if needed
    "location": "Properties > Material",
    "description": "Custom glTF material extensions for Bevy game development.",
    "category": "Import-Export",
}

import bpy

class AshiojinSandboxShaderParameters(bpy.types.PropertyGroup):
    param1: bpy.props.FloatVectorProperty(
        name="param1(RGBA)",
        description="Shader param1",
        size=4,
        subtype="COLOR",
        min=0,
        max=1,
        default=(1.0, 1.0, 1.0, 1.0)
    )

# 1. Define custom properties on the Material type
# This ensures every material data block inherently holds these variables.
def register_properties():
    bpy.utils.register_class(AshiojinSandboxShaderParameters)
    bpy.types.Material.ashiojin_shader_type = bpy.props.EnumProperty(
        name="Custom Shader Type",
        items=[
            ('None', "None", "Use StandardMaterial"),
            ('Sandbox', "Ashiojin Sandbox", "Use ExtendedMaterial of learn_shader.sandbox")
        ],
        default='None'
    )
    bpy.types.Material.ashiojin_sandbox_param = bpy.props.PointerProperty(type=AshiojinSandboxShaderParameters)
    

# 2. Create the Custom UI Panel
class PANEL_PT_my_engine_settings(bpy.types.Panel):
    bl_label = "My Engine Material Settings"
    bl_idname = "PANEL_PT_my_engine_settings"
    bl_space_type = 'PROPERTIES'
    bl_region_type = 'WINDOW'
    bl_context = "material" # Places it specifically in the Material properties tab

    def draw(self, context):
        layout = self.layout
        material = context.material
        
        if not material:
            layout.label(text="No Active Material")
            return

        # Draw the inputs directly mapped to the material's data variables
        layout.prop(material, "ashiojin_shader_type")
        
        # Only show wave speed if 'WATER' is selected (Conditional UI!)
        if material.ashiojin_shader_type == 'Sandbox':
            params = material.ashiojin_sandbox_param
            layout.prop(params, "param1")


# 3. Standard Blender Registration Boilerplate
classes = (PANEL_PT_my_engine_settings,)

fn_register, fn_unregister = bpy.utils.register_classes_factory(classes)

def register():
    register_properties()
    fn_register()

def unregister():
    fn_unregister()
    # Clean up properties on unregister
    del bpy.types.Material.ashiojin_sandbox_param
    del bpy.types.Material.ashiojin_shader_type

    bpy.utils.unregister_class(AshiojinSandboxShaderParameters)


# ====================================================================
# 4. GLTF EXPORTER EXTENSION HOOK
# ====================================================================

class glTF2ExportUserExtension:
    def __init__(self):
        print("extension init")
        # We need to import the official Extension wrapper dynamically
        from io_scene_gltf2.io.com.gltf2_io_extensions import Extension
        self.Extension = Extension

    def gather_material_hook(self, gltf2_material, blender_material, export_settings):
        print("gather_material_hook")
        """
        This function automatically triggers for EVERY material during export.
        """
        # Safety check: Ensure the material has our custom properties initialized
        if not hasattr(blender_material, "ashiojin_shader_type"):
            print("It has not ashiojin_shader_type")
            return

        # If the user selected 'None', we do nothing and let standard glTF handle it
        if blender_material.ashiojin_shader_type == 'None':
            print("ashiojin_shader_type is None")
            return

        # If the user chose 'Sandbox', we extract the nested parameters
        if blender_material.ashiojin_shader_type == 'Sandbox':
            print("ashiojin_shader_type is Sandbox")
            params = blender_material.ashiojin_sandbox_param
            
            # Note: Blender returns a Mathutils Vector/Color or array-like block.
            # We convert it into a standard Python list so it serializes cleanly to JSON.
            rgba_list = list(params.param1)

            # Construct your custom extension payload structured for your Bevy engine
            extension_payload = {
                "shader_type": "ASHIOJIN_SANDBOX",
                "param1": rgba_list
            }

            # Initialize the gltf extensions dictionary if it doesn't exist
            if gltf2_material.extensions is None:
                gltf2_material.extensions = {}

            # Assign your custom vendor namespace
            extension_name = "ASHIOJIN_material_sandbox"
            gltf2_material.extensions[extension_name] = self.Extension(
                name=extension_name,
                extension=extension_payload,
                required=False # Set to True if your engine absolutely requires it to render
            )    

if __name__ == "__main__":
    register()