import pygame

pygame.init()

# Set up the screen
screen = pygame.display.set_mode((1280, 720))
pygame.display.set_caption("Simple Level Editor")

half_width = screen.get_width() // 2
half_height = screen.get_height() // 2

# Initialize variables to store the coordinates
level = []
vertices = ''

drawing = False
start_point = None

LINE_COLOR = (255, 255, 255)
LINE_WIDTH = 2

cvt_coords(x:)

# Game loop
running = True
while running:
    for event in pygame.event.get():
        if event.type == pygame.QUIT:
            running = False
        elif event.type == pygame.MOUSEBUTTONDOWN:
            if event.button == 1:  # Left mouse button
                drawing = True
                start_point = event.pos
                vertices = 'l '
            elif event.button == 2:
                end_point = event.pot
                level.append(f"s {end_point[0] - half_width:.1f},{half_height - end_point[1]:.1f} ")
        elif event.type == pygame.MOUSEMOTION:
            if drawing and ((event.pos[0] - start_point[0]) ** 2 + (event.pos[1] - start_point[1]) ** 2) > 3:
                end_point = event.pos
                pygame.draw.line(screen, LINE_COLOR, start_point, end_point, LINE_WIDTH)
                vertices += f"{end_point[0] - half_width:.1f},{half_height - end_point[1]:.1f} "
                start_point = end_point
        elif event.type == pygame.MOUSEBUTTONUP:
            if event.button == 1:
                level.append(vertices)
                drawing = False
        elif event.type == pygame.KEYDOWN:
            if event.key == pygame.K_c:
                level.pop()
                # redraw the screen
                screen.fill((0, 0, 0))
                for line in level:
                    if line[:2] == 'l ':
                        vertices = line[2:].split()
                        start_point = (float(vertices[0].split(',')[0]) + half_width, - float(vertices[0].split(',')[1]) + half_height)
                        for vertex in vertices[1:]:
                            end_point = (float(vertex.split(',')[0]) + half_width, - float(vertex.split(',')[1]) + half_height)
                            pygame.draw.line(screen, LINE_COLOR, start_point, end_point, LINE_WIDTH)
                            start_point = end_point
                    elif line[:2] == 's ':
                        coords = lines[2:].split()
                        coor

    pygame.display.flip()

pygame.quit()

# Save the coordinates as JSON
with open("./src/levels/whataday.txt", "wb") as file:
    level = [s[:-1] for s in level]
    content = '\n'.join(level).encode(encoding='UTF-8')
    file.write(content)

