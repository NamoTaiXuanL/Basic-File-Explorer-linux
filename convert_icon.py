try:
    from PIL import Image
    import os

    # 读取PNG文件
    png_path = "material/png/logo_icon_0_150.png"
    ico_path = "material/png/logo_icon_0_150.ico"

    # 打开PNG图像
    img = Image.open(png_path)

    # 创建包含多个尺寸的图标列表
    sizes = [(16,16), (32,32), (48,48), (64,64), (128,128)]
    icons = []

    for size in sizes:
        # 调整大小并添加到列表
        resized_img = img.resize(size, Image.Resampling.LANCZOS)
        icons.append(resized_img)

    # 保存为ICO文件
    icons[0].save(ico_path, format='ICO', sizes=sizes)

    print(f"图标转换成功: {ico_path}")

except ImportError:
    print("需要安装Pillow库: pip install Pillow")
except Exception as e:
    print(f"转换失败: {e}")